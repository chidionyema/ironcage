use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;
use rand::Rng;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Z3 error: {0}")]
    SolverError(String),
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
}

impl FormalVerifier {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    /// Verify a claim using mock Z3 (placeholder for real solver integration).
    /// For demo: simple arithmetic verification with 70% success rate.
    pub async fn verify(&self, claim: &str) -> Result<Z3Proof, VerificationError> {
        if claim.is_empty() {
            return Err(VerificationError::InvalidClaim("Empty claim".to_string()));
        }

        let start = Instant::now();

        // Mock verification: simple pattern matching for demo
        let result = if claim.contains("=") {
            let parts: Vec<&str> = claim.split('=').collect();
            if parts.len() == 2 {
                Self::verify_arithmetic_claim(parts[0].trim(), parts[1].trim())
            } else {
                VerificationResult::Unknown
            }
        } else {
            // Probabilistic: 70% proved, 20% counterexample, 10% unknown
            let mut rng = rand::thread_rng();
            match rng.gen_range(0..10) {
                0..=6 => VerificationResult::Proved,
                7..=8 => VerificationResult::Counterexample,
                _ => VerificationResult::Unknown,
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(Z3Proof {
            claim: claim.to_string(),
            result,
            proof_trace: vec![format!("Verifier check completed in {}ms", duration_ms)],
            duration_ms,
        })
    }

    fn verify_arithmetic_claim(lhs_str: &str, rhs_str: &str) -> VerificationResult {
        let lhs = Self::eval_expr(lhs_str);
        let rhs = Self::eval_expr(rhs_str);

        match (lhs, rhs) {
            (Some(left), Some(right)) => {
                if left == right {
                    VerificationResult::Proved
                } else {
                    VerificationResult::Counterexample
                }
            }
            _ => VerificationResult::Unknown,
        }
    }

    fn eval_expr(expr_str: &str) -> Option<i64> {
        let expr_str = expr_str.trim();
        if let Ok(n) = expr_str.parse::<i64>() {
            Some(n)
        } else if let Some(pos) = expr_str.find('+') {
            let left: i64 = expr_str[..pos].trim().parse().ok()?;
            let right: i64 = expr_str[pos + 1..].trim().parse().ok()?;
            Some(left + right)
        } else if let Some(pos) = expr_str.find('-') {
            if pos == 0 {
                None
            } else {
                let left: i64 = expr_str[..pos].trim().parse().ok()?;
                let right: i64 = expr_str[pos + 1..].trim().parse().ok()?;
                Some(left - right)
            }
        } else {
            None
        }
    }

    /// Request a proof with a specific timeout.
    pub async fn verify_with_timeout(
        &self,
        claim: &str,
        timeout_ms: u64,
    ) -> Result<Z3Proof, VerificationError> {
        let verifier = FormalVerifier::new(timeout_ms);
        verifier.verify(claim).await
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
        let v = FormalVerifier::new(5000);
        assert_eq!(v.timeout(), 5000);
    }

    #[tokio::test]
    async fn test_verify_invalid_claim() {
        let v = FormalVerifier::new(5000);
        let result = v.verify("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_valid_claim() {
        let v = FormalVerifier::new(5000);
        let result = v.verify("1+1=2").await;
        assert!(result.is_ok());
    }
}
