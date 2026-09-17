//! Shared traits for DNA encoding/decoding

use crate::error::Result;

/// DNA encoding trait
pub trait DNACoder {
    /// Encode a message to DNA sequence
    fn encode_message(message: &str) -> Result<String>;

    /// Decode a DNA sequence to message
    fn decode_sequence(sequence: &str) -> Result<String>;
}

/// Sequence statistics trait
pub trait SequenceStats {
    /// Get statistics about a DNA sequence
    fn get_sequence_stats(sequence: &str) -> SequenceStatistics;
}

/// DNA sequence statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct SequenceStatistics {
    pub length: usize,
    pub bases: BaseCounts,
    pub gc_content: f64,
}

/// Base count statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct BaseCounts {
    pub a: usize,
    pub t: usize,
    pub c: usize,
    pub g: usize,
}

impl SequenceStatistics {
    /// Calculate statistics for a DNA sequence
    pub fn new(sequence: &str) -> Self {
        let length = sequence.len();
        let bases = BaseCounts::from_sequence(sequence);
        let gc_content = bases.calculate_gc_content(length);

        Self {
            length,
            bases,
            gc_content,
        }
    }
}

impl BaseCounts {
    /// Count bases from a sequence
    pub fn from_sequence(sequence: &str) -> Self {
        let mut a = 0;
        let mut t = 0;
        let mut c = 0;
        let mut g = 0;

        for ch in sequence.chars() {
            match ch.to_ascii_uppercase() {
                'A' => a += 1,
                'T' => t += 1,
                'C' => c += 1,
                'G' => g += 1,
                _ => {}
            }
        }

        Self { a, t, c, g }
    }

    /// Calculate GC content percentage
    pub fn calculate_gc_content(&self, total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }
        let gc_count = self.c + self.g;
        (gc_count as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_counts() {
        let counts = BaseCounts::from_sequence("ATCG");
        assert_eq!(counts.a, 1);
        assert_eq!(counts.t, 1);
        assert_eq!(counts.c, 1);
        assert_eq!(counts.g, 1);
    }

    #[test]
    fn test_gc_content() {
        let counts = BaseCounts {
            a: 2,
            t: 2,
            c: 2,
            g: 2,
        };
        assert_eq!(counts.calculate_gc_content(8), 50.0);

        let counts = BaseCounts {
            a: 4,
            t: 4,
            c: 0,
            g: 0,
        };
        assert_eq!(counts.calculate_gc_content(8), 0.0);

        let counts = BaseCounts {
            a: 0,
            t: 0,
            c: 4,
            g: 4,
        };
        assert_eq!(counts.calculate_gc_content(8), 100.0);
    }

    #[test]
    fn test_sequence_statistics() {
        let stats = SequenceStatistics::new("ATCG");
        assert_eq!(stats.length, 4);
        assert_eq!(stats.gc_content, 50.0);
        assert_eq!(stats.bases.a, 1);
        assert_eq!(stats.bases.t, 1);
        assert_eq!(stats.bases.c, 1);
        assert_eq!(stats.bases.g, 1);
    }
}
