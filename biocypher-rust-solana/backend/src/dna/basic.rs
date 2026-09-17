//! Basic DNA Cryptography Module
//!
//! Simple binary-to-DNA mapping: 00=A, 01=T, 10=C, 11=G
//! Native Rust port of the original BioCypher codec (see docs/LEGACY_LICENSE).

use crate::dna::traits::{DNACoder, SequenceStatistics, SequenceStats};
use crate::error::{DNACryptoError, Result};

/// Basic DNA cryptography encoder/decoder
pub struct DNACrypto;

impl DNACoder for DNACrypto {
    /// Encode a message to DNA sequence using basic mapping
    ///
    /// # Algorithm
    /// 1. Convert message to binary (8 bits per character)
    /// 2. Group binary in pairs
    /// 3. Map pairs to DNA bases: 00=A, 01=T, 10=C, 11=G
    /// 4. Return DNA sequence
    ///
    /// # Example
    /// ```
    /// use biocypher_backend::dna::basic::DNACrypto;
    /// use biocypher_backend::dna::traits::DNACoder;
    ///
    /// let dna = DNACrypto::encode_message("Hi").unwrap();
    /// assert_eq!(dna, "TACATCCT");
    /// ```
    fn encode_message(message: &str) -> Result<String> {
        if message.is_empty() {
            return Ok(String::new());
        }

        let binary = Self::text_to_binary(message)?;
        let dna = Self::binary_to_dna(&binary)?;

        Ok(dna)
    }

    /// Decode a DNA sequence back to the original message
    ///
    /// # Algorithm
    /// 1. Convert DNA sequence to binary (A=00, T=01, C=10, G=11)
    /// 2. Group binary in 8-bit chunks
    /// 3. Convert each byte to ASCII character
    /// 4. Return message
    ///
    /// # Example
    /// ```
    /// use biocypher_backend::dna::basic::DNACrypto;
    /// use biocypher_backend::dna::traits::DNACoder;
    ///
    /// let message = DNACrypto::decode_sequence("TACATCCT").unwrap();
    /// assert_eq!(message, "Hi");
    /// ```
    fn decode_sequence(sequence: &str) -> Result<String> {
        if sequence.is_empty() {
            return Ok(String::new());
        }

        let binary = Self::dna_to_binary(sequence)?;
        let text = Self::binary_to_text(&binary)?;

        Ok(text)
    }
}

impl SequenceStats for DNACrypto {
    /// Get statistics about a DNA sequence
    fn get_sequence_stats(sequence: &str) -> SequenceStatistics {
        SequenceStatistics::new(sequence)
    }
}

impl DNACrypto {
    /// DNA base encoding mapping
    const DNA_ENCODE: [(u8, &str); 4] = [(0b00, "A"), (0b01, "T"), (0b10, "C"), (0b11, "G")];

    /// Convert text to binary representation
    ///
    /// Each character is converted to its ASCII value (0-255),
    /// then to an 8-bit binary string.
    fn text_to_binary(text: &str) -> Result<String> {
        let mut binary = String::with_capacity(text.len() * 8);

        for byte in text.as_bytes() {
            binary.push_str(&format!("{:08b}", byte));
        }

        Ok(binary)
    }

    /// Convert binary string to DNA sequence
    ///
    /// Binary is processed in pairs (2 bits → 1 base).
    /// If binary length is odd, pad with '0' for the final pair.
    fn binary_to_dna(binary: &str) -> Result<String> {
        if binary.is_empty() {
            return Ok(String::new());
        }

        let mut dna = String::with_capacity(binary.len() / 2);
        let chars: Vec<char> = binary.chars().collect();

        for chunk in chars.chunks(2) {
            let pair = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                // Pad with '0' for odd-length binary
                format!("{}{}", chunk[0], '0')
            };

            let bits = u8::from_str_radix(&pair, 2)
                .map_err(|e| DNACryptoError::InvalidBinary(e.to_string()))?;

            let base = Self::DNA_ENCODE
                .iter()
                .find(|(b, _)| *b == bits)
                .map(|(_, s)| *s)
                .ok_or_else(|| DNACryptoError::InvalidBinaryPair(pair.clone()))?;

            dna.push_str(base);
        }

        Ok(dna)
    }

    /// Convert DNA sequence to binary string
    ///
    /// Invalid bases (not A, T, C, G) are skipped.
    fn dna_to_binary(dna: &str) -> Result<String> {
        if dna.is_empty() {
            return Ok(String::new());
        }

        let mut binary = String::with_capacity(dna.len() * 2);

        for base in dna.chars() {
            if let Some(bits) = match base.to_ascii_uppercase() {
                'A' => Some("00"),
                'T' => Some("01"),
                'C' => Some("10"),
                'G' => Some("11"),
                _ => None,
            } {
                binary.push_str(bits);
            }
            // Skip invalid bases
        }

        Ok(binary)
    }

    /// Convert binary string to text
    ///
    /// Binary is processed in 8-bit chunks (bytes).
    /// Preserve UTF-8 and whitespace so imported text files round-trip intact.
    fn binary_to_text(binary: &str) -> Result<String> {
        if binary.is_empty() {
            return Ok(String::new());
        }

        let mut bytes = Vec::new();
        let chars: Vec<char> = binary.chars().collect();

        for chunk in chars.chunks(8) {
            if chunk.len() == 8 {
                let byte_str: String = chunk.iter().collect();
                let byte_val = u8::from_str_radix(&byte_str, 2)
                    .map_err(|e| DNACryptoError::InvalidBinary(e.to_string()))?;

                bytes.push(byte_val);
            }
        }

        String::from_utf8(bytes).map_err(|e| {
            DNACryptoError::DecodingFailed(format!("Decoded bytes are not UTF-8: {e}")).into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple() {
        let dna = DNACrypto::encode_message("A").unwrap();
        // 'A' = 65 = 01000001
        // 01 00 00 01 = T A A T
        assert_eq!(dna, "TAAT");
    }

    #[test]
    fn test_encode_hello() {
        let dna = DNACrypto::encode_message("Hello").unwrap();
        assert!(!dna.is_empty());
        assert!(dna.len() % 4 == 0); // Should be multiple of 4
    }

    #[test]
    fn test_decode_simple() {
        let message = DNACrypto::decode_sequence("TAAT").unwrap();
        assert_eq!(message, "A");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = "Hello, World!";
        let dna = DNACrypto::encode_message(original).unwrap();
        let decoded = DNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(DNACrypto::encode_message("").unwrap(), "");
        assert_eq!(DNACrypto::decode_sequence("").unwrap(), "");
    }

    #[test]
    fn test_special_characters() {
        let original = "Test @#$%";
        let dna = DNACrypto::encode_message(original).unwrap();
        let decoded = DNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_newline_characters() {
        let original = "Line 1\nLine 2";
        let dna = DNACrypto::encode_message(original).unwrap();
        let decoded = DNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_invalid_bases_skipped() {
        // "TAAT" = 'A' (65), invalid 'X' and space skipped
        let dna_with_invalid = "TAAT XCGT";
        let decoded = DNACrypto::decode_sequence(dna_with_invalid).unwrap();
        // Invalid 'X' and space skipped; TAAT = 'A', CGT = 6 bits (incomplete byte)
        assert_eq!(decoded, "A");
    }

    #[test]
    fn test_binary_mapping() {
        // Test each mapping
        assert_eq!(DNACrypto::binary_to_dna("00").unwrap(), "A");
        assert_eq!(DNACrypto::binary_to_dna("01").unwrap(), "T");
        assert_eq!(DNACrypto::binary_to_dna("10").unwrap(), "C");
        assert_eq!(DNACrypto::binary_to_dna("11").unwrap(), "G");
    }

    #[test]
    fn test_reverse_mapping() {
        assert_eq!(DNACrypto::dna_to_binary("A").unwrap(), "00");
        assert_eq!(DNACrypto::dna_to_binary("T").unwrap(), "01");
        assert_eq!(DNACrypto::dna_to_binary("C").unwrap(), "10");
        assert_eq!(DNACrypto::dna_to_binary("G").unwrap(), "11");
    }

    #[test]
    fn test_text_to_binary() {
        let binary = DNACrypto::text_to_binary("A").unwrap();
        assert_eq!(binary, "01000001");

        let binary = DNACrypto::text_to_binary("Hi").unwrap();
        assert_eq!(binary.len(), 16); // 2 bytes * 8 bits
    }

    #[test]
    fn test_binary_to_text() {
        let text = DNACrypto::binary_to_text("01000001").unwrap();
        assert_eq!(text, "A");
    }

    #[test]
    fn test_sequence_stats() {
        let stats = DNACrypto::get_sequence_stats("ATCG");
        assert_eq!(stats.length, 4);
        assert_eq!(stats.bases.a, 1);
        assert_eq!(stats.bases.t, 1);
        assert_eq!(stats.bases.c, 1);
        assert_eq!(stats.bases.g, 1);
        assert_eq!(stats.gc_content, 50.0);
    }

    #[test]
    fn test_gc_content_calculation() {
        let stats = DNACrypto::get_sequence_stats("AAAA");
        assert_eq!(stats.gc_content, 0.0);

        let stats = DNACrypto::get_sequence_stats("CCCC");
        assert_eq!(stats.gc_content, 100.0);

        let stats = DNACrypto::get_sequence_stats("ACGTACGT");
        assert_eq!(stats.gc_content, 50.0);
    }

    #[test]
    fn test_case_insensitive_decoding() {
        let dna_upper = "TAAA";
        let dna_lower = "taaa";
        let dna_mixed = "TaAa";

        assert_eq!(
            DNACrypto::decode_sequence(dna_upper).unwrap(),
            DNACrypto::decode_sequence(dna_lower).unwrap()
        );
        assert_eq!(
            DNACrypto::decode_sequence(dna_upper).unwrap(),
            DNACrypto::decode_sequence(dna_mixed).unwrap()
        );
    }

    #[test]
    fn test_long_message() {
        let original = "The quick brown fox jumps over the lazy dog";
        let dna = DNACrypto::encode_message(original).unwrap();
        let decoded = DNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_all_printable_ascii() {
        let mut all_chars = String::new();
        for i in 32..=126u8 {
            all_chars.push(i as char);
        }

        let dna = DNACrypto::encode_message(&all_chars).unwrap();
        let decoded = DNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(all_chars, decoded);
    }
}
