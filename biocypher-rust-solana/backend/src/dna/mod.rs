//! DNA Cryptography Module
//!
//! Handles encoding and decoding of messages using DNA bases with support for
//! three modes: Basic, Nanopore, and Secure.

pub mod basic;
pub mod nanopore;
pub mod secure;
pub mod split_key;
pub mod traits;

pub use basic::DNACrypto;
pub use nanopore::NanoporeDNACrypto;
pub use secure::SecureDNACrypto;
pub use split_key::SplitKeyDNACrypto;
pub use traits::*;

/// DNA base representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DNABase {
    A,
    T,
    C,
    G,
}

impl DNABase {
    /// Convert base to char
    pub fn as_char(&self) -> char {
        match self {
            DNABase::A => 'A',
            DNABase::T => 'T',
            DNABase::C => 'C',
            DNABase::G => 'G',
        }
    }

    /// Convert char to base
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'A' | 'a' => Some(DNABase::A),
            'T' | 't' => Some(DNABase::T),
            'C' | 'c' => Some(DNABase::C),
            'G' | 'g' => Some(DNABase::G),
            _ => None,
        }
    }

    /// Convert to binary pair
    pub fn to_binary_pair(&self) -> &'static str {
        match self {
            DNABase::A => "00",
            DNABase::T => "01",
            DNABase::C => "10",
            DNABase::G => "11",
        }
    }

    /// Convert from binary pair
    pub fn from_binary_pair(bits: &str) -> Option<Self> {
        match bits {
            "00" => Some(DNABase::A),
            "01" => Some(DNABase::T),
            "10" => Some(DNABase::C),
            "11" => Some(DNABase::G),
            _ => None,
        }
    }
}

/// Encoding mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncodingMode {
    Basic,
    Nanopore,
    Secure,
    SplitKey,
}

impl std::fmt::Display for EncodingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingMode::Basic => write!(f, "basic"),
            EncodingMode::Nanopore => write!(f, "nanopore"),
            EncodingMode::Secure => write!(f, "secure"),
            EncodingMode::SplitKey => write!(f, "splitkey"),
        }
    }
}

impl std::str::FromStr for EncodingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(EncodingMode::Basic),
            "nanopore" => Ok(EncodingMode::Nanopore),
            "secure" => Ok(EncodingMode::Secure),
            "splitkey" => Ok(EncodingMode::SplitKey),
            _ => Err(format!("Invalid encoding mode: {}", s)),
        }
    }
}

/// Nanopore markers
pub mod markers {
    /// Start marker for nanopore sequences
    pub const START_MARKER: &str = "ATCGATCG";
    /// Stop marker for nanopore sequences
    pub const STOP_MARKER: &str = "CGATATCG";
    /// Padding delimiter
    pub const PADDING_DELIMITER: &str = "TACGTA";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dna_base_conversions() {
        assert_eq!(DNABase::A.as_char(), 'A');
        assert_eq!(DNABase::from_char('A'), Some(DNABase::A));
        assert_eq!(DNABase::A.to_binary_pair(), "00");
        assert_eq!(DNABase::from_binary_pair("00"), Some(DNABase::A));
    }

    #[test]
    fn test_encoding_mode() {
        assert_eq!(EncodingMode::Basic.to_string(), "basic");
        assert_eq!(EncodingMode::Nanopore.to_string(), "nanopore");
        assert_eq!(EncodingMode::Secure.to_string(), "secure");
        assert_eq!(EncodingMode::SplitKey.to_string(), "splitkey");

        assert_eq!("basic".parse::<EncodingMode>(), Ok(EncodingMode::Basic));
        assert_eq!(
            "nanopore".parse::<EncodingMode>(),
            Ok(EncodingMode::Nanopore)
        );
        assert_eq!("secure".parse::<EncodingMode>(), Ok(EncodingMode::Secure));
        assert_eq!(
            "splitkey".parse::<EncodingMode>(),
            Ok(EncodingMode::SplitKey)
        );
    }
}
