//! Nanopore-Optimized DNA Cryptography Module
//!
//! Designed specifically for nanopore sequencing constraints
//! Ported from Python: biocypher/nanopore_dna_crypto.py

use crate::dna::markers;
use crate::dna::traits::{DNACoder, SequenceStatistics, SequenceStats};
use crate::error::{DNACryptoError, Result};
use regex::Regex;

/// Nanopore-optimized DNA cryptography with error correction and homopolymer avoidance
pub struct NanoporeDNACrypto;

impl DNACoder for NanoporeDNACrypto {
    /// Encode message with nanopore optimizations
    fn encode_message(message: &str) -> Result<String> {
        Self::encode_message_with_options(message, true)
    }

    /// Decode nanopore DNA sequence
    fn decode_sequence(sequence: &str) -> Result<String> {
        Self::decode_sequence_with_options(sequence, true)
    }
}

impl SequenceStats for NanoporeDNACrypto {
    fn get_sequence_stats(sequence: &str) -> SequenceStatistics {
        SequenceStatistics::new(sequence)
    }
}

impl NanoporeDNACrypto {
    /// Triplet encoding table (avoids homopolymers)
    const NANOPORE_ENCODE: [(&'static str, &'static str); 8] = [
        ("000", "ATC"),
        ("001", "ATG"),
        ("010", "ACT"),
        ("011", "ACG"),
        ("100", "TAG"),
        ("101", "TAC"),
        ("110", "TCG"),
        ("111", "TCA"),
    ];

    /// Reverse mapping for decoding
    fn triplet_to_bits(triplet: &str) -> Option<&'static str> {
        let upper = triplet.to_uppercase();
        Self::NANOPORE_ENCODE
            .iter()
            .find(|(_, dna)| *dna == upper)
            .map(|(bits, _)| *bits)
    }

    /// Optimal GC content range for nanopore
    const OPTIMAL_GC_MIN: f64 = 40.0;
    const OPTIMAL_GC_MAX: f64 = 60.0;

    /// Error correction repeats
    const ERROR_CORRECTION_REPEATS: usize = 3;

    /// Encode with optional error correction
    pub fn encode_message_with_options(
        message: &str,
        use_error_correction: bool,
    ) -> Result<String> {
        if message.is_empty() {
            return Ok(String::new());
        }

        // Step 1: Text to binary with parity (9 bits per char)
        let binary = Self::text_to_binary_with_parity(message)?;

        // Step 2: Add error correction (triple redundancy)
        let binary = if use_error_correction {
            Self::add_error_correction(&binary)
        } else {
            binary
        };

        // Step 3: Convert to DNA with triplet encoding
        let mut dna = Self::binary_to_nanopore_dna(&binary)?;

        // Step 4: Add padding if homopolymers or GC unbalanced
        if Self::has_homopolymers(&dna) || !Self::is_gc_balanced(&dna) {
            let padding = Self::generate_nanopore_padding(&dna);
            dna = format!(
                "{}{}{}{}{}",
                padding,
                markers::PADDING_DELIMITER,
                dna,
                markers::PADDING_DELIMITER,
                padding
            );
        }

        // Step 5: Add nanopore markers
        dna = format!("{}{}{}", markers::START_MARKER, dna, markers::STOP_MARKER);

        Ok(dna)
    }

    /// Decode with optional error correction
    pub fn decode_sequence_with_options(
        sequence: &str,
        use_error_correction: bool,
    ) -> Result<String> {
        if sequence.is_empty() {
            return Ok(String::new());
        }

        // Step 1: Remove markers
        let mut seq = Self::remove_nanopore_markers(sequence);

        // Step 2: Remove padding if present
        seq = Self::remove_nanopore_padding(&seq);

        // Step 3: Convert DNA to binary
        let mut binary = Self::nanopore_dna_to_binary(&seq)?;

        // Step 4: Error correction (majority voting)
        if use_error_correction {
            binary = Self::correct_errors(&binary)?;
        }

        // Step 5: Binary to text with parity check
        let text = Self::binary_to_text_with_parity(&binary)?;

        Ok(text)
    }

    /// Text to binary with parity bit (9 bits per char: 8 data + 1 parity)
    fn text_to_binary_with_parity(text: &str) -> Result<String> {
        let mut binary = String::with_capacity(text.len() * 9);
        for byte in text.as_bytes() {
            let bin_byte = format!("{:08b}", byte);
            let parity = (bin_byte.chars().filter(|c| *c == '1').count() % 2) as u8;
            binary.push_str(&bin_byte);
            binary.push(if parity == 0 { '0' } else { '1' });
        }
        Ok(binary)
    }

    /// Add triple redundancy for error correction
    fn add_error_correction(binary: &str) -> String {
        binary
            .chars()
            .flat_map(|c| std::iter::repeat_n(c, Self::ERROR_CORRECTION_REPEATS))
            .collect()
    }

    /// Convert binary to nanopore DNA (triplet encoding)
    fn binary_to_nanopore_dna(binary: &str) -> Result<String> {
        let mut padded = binary.to_string();
        while !padded.len().is_multiple_of(3) {
            padded.push('0');
        }

        let mut dna = String::with_capacity(padded.len() / 3 * 3);
        for chunk in padded.as_bytes().chunks(3) {
            let triplet: String = chunk.iter().map(|b| *b as char).collect();
            let base = Self::NANOPORE_ENCODE
                .iter()
                .find(|(bits, _)| *bits == triplet)
                .map(|(_, s)| *s)
                .ok_or_else(|| DNACryptoError::InvalidBinary(triplet.clone()))?;
            dna.push_str(base);
        }
        Ok(dna)
    }

    /// Check for homopolymer runs (2+ consecutive same base)
    fn has_homopolymers(sequence: &str) -> bool {
        for base in ['A', 'T', 'C', 'G'] {
            let pattern = format!("{}{{2,}}", base);
            if Regex::new(&pattern).is_ok_and(|re| re.is_match(sequence)) {
                return true;
            }
        }
        false
    }

    /// Check if GC content is in optimal range
    fn is_gc_balanced(sequence: &str) -> bool {
        if sequence.is_empty() {
            return true;
        }
        let gc_count = sequence
            .chars()
            .filter(|c| matches!(c.to_ascii_uppercase(), 'G' | 'C'))
            .count();
        let gc_content = (gc_count as f64 / sequence.len() as f64) * 100.0;
        (Self::OPTIMAL_GC_MIN..=Self::OPTIMAL_GC_MAX).contains(&gc_content)
    }

    /// Generate nanopore-friendly padding
    fn generate_nanopore_padding(sequence: &str) -> String {
        const PADDING_PATTERN: &str = "ATCATGACTACG";
        let padding_length = (sequence.len() / 10).max(6);
        let full_repeats = padding_length / PADDING_PATTERN.len();
        let remainder = padding_length % PADDING_PATTERN.len();
        let mut padding = PADDING_PATTERN.repeat(full_repeats);
        padding.push_str(&PADDING_PATTERN[..remainder]);
        padding
    }

    /// Remove start/stop markers
    fn remove_nanopore_markers(sequence: &str) -> String {
        let mut seq = sequence.to_string();
        if seq.starts_with(markers::START_MARKER) {
            seq = seq[markers::START_MARKER.len()..].to_string();
        }
        if seq.ends_with(markers::STOP_MARKER) {
            seq = seq[..seq.len() - markers::STOP_MARKER.len()].to_string();
        }
        seq
    }

    /// Remove padding between delimiters
    fn remove_nanopore_padding(sequence: &str) -> String {
        let first = sequence.find(markers::PADDING_DELIMITER);
        let last = sequence.rfind(markers::PADDING_DELIMITER);

        if let (Some(first_pos), Some(last_pos)) = (first, last) {
            if first_pos != last_pos {
                let start = first_pos + markers::PADDING_DELIMITER.len();
                let end = last_pos;
                if start < end {
                    return sequence[start..end].to_string();
                }
            }
        }
        sequence.to_string()
    }

    /// Majority voting for error correction
    fn correct_errors(binary: &str) -> Result<String> {
        let mut padded = binary.to_string();
        let r = Self::ERROR_CORRECTION_REPEATS;
        while !padded.len().is_multiple_of(r) {
            padded.push('0');
        }

        let mut corrected = String::with_capacity(padded.len() / r);
        for chunk in padded.as_bytes().chunks(r) {
            let ones = chunk.iter().filter(|b| **b == b'1').count();
            let zeros = chunk.iter().filter(|b| **b == b'0').count();
            corrected.push(if ones > zeros { '1' } else { '0' });
        }
        Ok(corrected)
    }

    /// Convert nanopore DNA to binary
    fn nanopore_dna_to_binary(dna: &str) -> Result<String> {
        let mut binary = String::new();
        let chars: Vec<char> = dna.chars().collect();
        for chunk in chars.chunks(3) {
            if chunk.len() == 3 {
                let triplet: String = chunk.iter().collect();
                if let Some(bits) = Self::triplet_to_bits(&triplet) {
                    binary.push_str(bits);
                }
            }
        }
        Ok(binary)
    }

    /// Binary to text with parity check (9 bits per char)
    fn binary_to_text_with_parity(binary: &str) -> Result<String> {
        let mut bytes = Vec::new();
        let chars: Vec<char> = binary.chars().collect();
        for chunk in chars.chunks(9) {
            if chunk.len() == 9 {
                let data_bits: String = chunk[..8].iter().collect();
                let parity_bit = chunk[8];
                let expected_parity = if data_bits.chars().filter(|c| *c == '1').count() % 2 == 0 {
                    '0'
                } else {
                    '1'
                };
                if parity_bit == expected_parity {
                    if let Ok(byte_val) = u8::from_str_radix(&data_bits, 2) {
                        bytes.push(byte_val);
                    }
                }
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
    fn test_triplet_encoding_table() {
        assert_eq!(NanoporeDNACrypto::NANOPORE_ENCODE[0].1, "ATC");
        assert_eq!(NanoporeDNACrypto::NANOPORE_ENCODE[7].1, "TCA");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = "Hello";
        let dna = NanoporeDNACrypto::encode_message(original).unwrap();
        assert!(dna.starts_with(markers::START_MARKER));
        assert!(dna.ends_with(markers::STOP_MARKER));
        let decoded = NanoporeDNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_decode_empty() {
        assert_eq!(NanoporeDNACrypto::encode_message("").unwrap(), "");
        assert_eq!(NanoporeDNACrypto::decode_sequence("").unwrap(), "");
    }

    #[test]
    fn test_encode_decode_simple() {
        let original = "A";
        let dna = NanoporeDNACrypto::encode_message(original).unwrap();
        let decoded = NanoporeDNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_decode_long() {
        let original = "The quick brown fox jumps over the lazy dog";
        let dna = NanoporeDNACrypto::encode_message(original).unwrap();
        let decoded = NanoporeDNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_decode_special_chars() {
        let original = "Test @#$%^&*()";
        let dna = NanoporeDNACrypto::encode_message(original).unwrap();
        let decoded = NanoporeDNACrypto::decode_sequence(&dna).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_without_error_correction() {
        let original = "Hi";
        let dna = NanoporeDNACrypto::encode_message_with_options(original, false).unwrap();
        let decoded = NanoporeDNACrypto::decode_sequence_with_options(&dna, false).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_has_homopolymers() {
        assert!(NanoporeDNACrypto::has_homopolymers("AATC"));
        assert!(NanoporeDNACrypto::has_homopolymers("ATTT"));
        assert!(!NanoporeDNACrypto::has_homopolymers("ATCG"));
    }

    #[test]
    fn test_is_gc_balanced() {
        assert!(NanoporeDNACrypto::is_gc_balanced("ATCG")); // 50% GC
        assert!(NanoporeDNACrypto::is_gc_balanced("ATCGATCG")); // 50% GC
    }
}
