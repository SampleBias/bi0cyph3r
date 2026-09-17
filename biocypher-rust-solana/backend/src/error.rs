//! Error types for BioCypher backend

use thiserror::Error;

/// Result type alias for BioCypher operations
pub type Result<T> = std::result::Result<T, BioCypherError>;

/// Main error type for BioCypher backend
#[derive(Error, Debug)]
pub enum BioCypherError {
    // DNA crypto errors
    #[error("DNA crypto error: {0}")]
    DNACrypto(#[from] DNACryptoError),

    // Safety screener errors
    #[error("Safety screener error: {0}")]
    SafetyScreener(#[from] SafetyScreenerError),

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    // Solana errors (for Phase 2)
    #[error("Solana error: {0}")]
    Solana(String),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Generic errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// DNA crypto specific errors
#[derive(Error, Debug, Clone)]
pub enum DNACryptoError {
    #[error("Invalid binary pair: {0}")]
    InvalidBinaryPair(String),

    #[error("Invalid binary string: {0}")]
    InvalidBinary(String),

    #[error("Invalid DNA sequence: {0}")]
    InvalidSequence(String),

    #[error("Missing required markers")]
    MissingMarkers,

    #[error("Decoding failed: {0}")]
    DecodingFailed(String),

    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Password required for secure mode")]
    PasswordRequired,

    #[error("K1 and K2 required for split-key mode")]
    SplitKeyRequired,

    #[error("Password too weak: {0}")]
    PasswordWeak(String),
}

/// Safety screener specific errors
#[derive(Error, Debug, Clone)]
pub enum SafetyScreenerError {
    #[error("Empty sequence provided")]
    EmptySequence,

    #[error("No valid DNA bases found in sequence")]
    NoValidBases,

    #[error("Pathogen detection error: {0}")]
    PathogenDetectionError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = DNACryptoError::InvalidBinaryPair("00".to_string());
        assert_eq!(error.to_string(), "Invalid binary pair: 00");
    }

    #[test]
    fn test_safety_screener_error() {
        let error = SafetyScreenerError::EmptySequence;
        assert_eq!(error.to_string(), "Empty sequence provided");
    }

    #[test]
    fn test_error_conversion() {
        let dna_error: BioCypherError = DNACryptoError::PasswordRequired.into();
        assert!(matches!(dna_error, BioCypherError::DNACrypto(_)));
    }
}
