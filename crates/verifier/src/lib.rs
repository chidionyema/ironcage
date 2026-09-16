use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Z3 server unreachable: {0}")]
    ServerError(String),
    #[error("Verification timeout")]
    Timeout,
    #[error("Invalid claim: {0}")]
    InvalidClaim(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VerificationResult {
    Proved,           // Claim is valid (unsat negation)
    Counterexample,   // Claim is false (sat found)
    Unknown,          // Could not determine
    Timeout,          // Exceeded time limit
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Z3Proof {
    pub claim: String,
    pub result: VerificationResult,
    pub proof_trace: Vec<String>,
    pub duration_ms: u64,
}

pub struct FormalVerifier {
    timeout_ms: u64,
    server_url: String,
}

impl FormalVerifier {
    pub fn new(server_url: String, timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            server_url,
        }
    }

    /// Verify a claim using Z3 via MCP.
    pub async fn verify(&self, claim: &str) -> Result<Z3Proof, VerificationError> {
        if claim.is_empty() {
            return Err(VerificationError::InvalidClaim("Empty claim".to_string()));
        }

        // Placeholder: In production, this connects to Z3 MCP server and sends the claim
        // The server returns:
        // - sat: counterexample found (claim is false)
        // - unsat: claim proven true
        // - unknown: undecidable

        let result = VerificationResult::Unknown; // Stub
        let proof_trace = vec!["Z3 solver output...".to_string()];

        Ok(Z3Proof {
            claim: claim.to_string(),
            result,
            proof_trace,
            duration_ms: 0,
        })
    }

    /// Request a proof with a specific timeout.
    pub async fn verify_with_timeout(
        &self,
        claim: &str,
        timeout_ms: u64,
    ) -> Result<Z3Proof, VerificationError> {
        let verifier = FormalVerifier::new(self.server_url.clone(), timeout_ms);
        verifier.verify(claim).await
    }

    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    pub fn timeout(&self) -> u64 {
        self.timeout_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result_proved() {
        assert_eq!(
            VerificationResult::Proved,
            VerificationResult::Proved
        );
    }

    #[test]
    fn test_proof_construction() {
        let proof = Z3Proof {
            claim: "P = NP".to_string(),
            result: VerificationResult::Unknown,
            proof_trace: vec!["step 1".to_string()],
            duration_ms: 100,
        };
        assert!(!proof.claim.is_empty());
    }

    #[test]
    fn test_verifier_creation() {
        let v = FormalVerifier::new("http://z3-mcp:9999".to_string(), 5000);
        assert_eq!(v.timeout(), 5000);
    }

    #[tokio::test]
    async fn test_verify_invalid_claim() {
        let v = FormalVerifier::new("http://localhost:9999".to_string(), 5000);
        let result = v.verify("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_valid_claim() {
        let v = FormalVerifier::new("http://localhost:9999".to_string(), 5000);
        let result = v.verify("1+1=2").await;
        assert!(result.is_ok());
    }
}
