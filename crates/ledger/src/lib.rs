use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),
    #[error("Chain verification failed: {0}")]
    VerificationFailed(String),
    #[error("Invalid entry: {0}")]
    InvalidEntry(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub payload: String,
    pub prev_hash: String,
    pub hash: String,
}

impl LedgerEntry {
    fn compute_hash(prev_hash: &str, event_type: &str, payload: &str, timestamp: u64) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(event_type.as_bytes());
        hasher.update(payload.as_bytes());
        hasher.update(timestamp.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn verify_hash(&self) -> bool {
        let computed = Self::compute_hash(&self.prev_hash, &self.event_type, &self.payload, self.timestamp);
        computed == self.hash
    }
}

pub struct Ledger {
    conn: Connection,
}

impl Ledger {
    pub fn open(path: &str) -> Result<Self, LedgerError> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for durability (BABYLON-60 pattern)
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             PRAGMA foreign_keys=ON;",
        )?;

        // Create ledger table (append-only)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ledger (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                prev_hash TEXT NOT NULL,
                hash TEXT NOT NULL
            )",
            [],
        )?;

        // Enforce immutability via triggers
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS trg_ledger_immutable_update
                BEFORE UPDATE ON ledger
                BEGIN SELECT RAISE(ABORT, 'ledger is append-only'); END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS trg_ledger_immutable_delete
                BEFORE DELETE ON ledger
                BEGIN SELECT RAISE(ABORT, 'ledger is append-only'); END",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn append(
        &self,
        event_type: String,
        payload: String,
    ) -> Result<LedgerEntry, LedgerError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let prev_hash: String = self
            .conn
            .query_row(
                "SELECT hash FROM ledger ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "genesis".to_string());

        let hash = LedgerEntry::compute_hash(&prev_hash, &event_type, &payload, timestamp);

        self.conn.execute(
            "INSERT INTO ledger (timestamp, event_type, payload, prev_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![timestamp, event_type, payload, prev_hash, hash],
        )?;

        let id = self.conn.last_insert_rowid() as u64;

        Ok(LedgerEntry {
            id,
            timestamp,
            event_type,
            payload,
            prev_hash,
            hash,
        })
    }

    pub fn head(&self) -> Result<Option<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, payload, prev_hash, hash
             FROM ledger ORDER BY id DESC LIMIT 1",
        )?;

        let entry = stmt.query_row([], |row| {
            Ok(LedgerEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                payload: row.get(3)?,
                prev_hash: row.get(4)?,
                hash: row.get(5)?,
            })
        });

        match entry {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LedgerError::DbError(e)),
        }
    }

    pub fn verify_chain(&self) -> Result<bool, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, payload, prev_hash, hash
             FROM ledger ORDER BY id ASC",
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(LedgerEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                payload: row.get(3)?,
                prev_hash: row.get(4)?,
                hash: row.get(5)?,
            })
        })?;

        let mut prev_hash = "genesis".to_string();

        for entry_result in entries {
            let entry = entry_result?;

            if entry.prev_hash != prev_hash {
                return Err(LedgerError::VerificationFailed(
                    format!("Entry {} has invalid prev_hash", entry.id),
                ));
            }

            if !entry.verify_hash() {
                return Err(LedgerError::VerificationFailed(
                    format!("Entry {} hash mismatch", entry.id),
                ));
            }

            prev_hash = entry.hash;
        }

        Ok(true)
    }

    pub fn all_entries(&self) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, payload, prev_hash, hash
             FROM ledger ORDER BY id ASC",
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(LedgerEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                payload: row.get(3)?,
                prev_hash: row.get(4)?,
                hash: row.get(5)?,
            })
        })?;

        let mut result = Vec::new();
        for entry_result in entries {
            result.push(entry_result?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_ledger_creation() {
        let path = "/tmp/test_ledger.db";
        let _ = fs::remove_file(path);

        let ledger = Ledger::open(path).unwrap();
        let head = ledger.head().unwrap();
        assert!(head.is_none());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_ledger_append() {
        let path = "/tmp/test_ledger_append.db";
        let _ = fs::remove_file(path);

        let ledger = Ledger::open(path).unwrap();
        let entry = ledger
            .append("hypothesis_verified".to_string(), "P=NP".to_string())
            .unwrap();

        assert_eq!(entry.event_type, "hypothesis_verified");
        assert_eq!(entry.id, 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_chain_verification() {
        let path = "/tmp/test_chain.db";
        let _ = fs::remove_file(path);

        let ledger = Ledger::open(path).unwrap();
        ledger
            .append("event1".to_string(), "payload1".to_string())
            .unwrap();
        ledger
            .append("event2".to_string(), "payload2".to_string())
            .unwrap();

        let verified = ledger.verify_chain().unwrap();
        assert!(verified);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_entry_hash_verification() {
        let entry = LedgerEntry {
            id: 1,
            timestamp: 1000,
            event_type: "test".to_string(),
            payload: "data".to_string(),
            prev_hash: "genesis".to_string(),
            hash: LedgerEntry::compute_hash("genesis", "test", "data", 1000),
        };
        assert!(entry.verify_hash());
    }
}
