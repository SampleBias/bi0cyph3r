//! Data models for the local safety screener.

use serde::{Deserialize, Serialize};

/// Safety status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SafetyStatus {
    Safe,
    Caution,
    Unsafe,
}

/// Safety status icon mapping
impl SafetyStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            SafetyStatus::Safe => "✅",
            SafetyStatus::Caution => "⚠️",
            SafetyStatus::Unsafe => "❌",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            SafetyStatus::Safe => "green",
            SafetyStatus::Caution => "orange",
            SafetyStatus::Unsafe => "red",
        }
    }
}

/// Pathogen analysis result
#[derive(Debug, Clone, Serialize)]
pub struct PathogenAnalysis {
    /// Pathogen risk detected
    pub pathogen_risk: bool,

    /// Matching signatures
    pub matches: Vec<PathogenMatch>,

    /// Risk level
    pub risk_level: RiskLevel,
}

/// Pathogen signature match
#[derive(Debug, Clone, Serialize)]
pub struct PathogenMatch {
    /// Category of pathogen
    pub category: String,

    /// Matching signature
    pub signature: String,

    /// Position in sequence
    pub position: usize,
}

/// Risk level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Natural occurrence result
#[derive(Debug, Clone, Serialize)]
pub struct NaturalOccurrence {
    /// Natural occurrence detected
    pub natural_occurrence: bool,

    /// Matching signatures
    pub matches: Vec<NaturalMatch>,

    /// Organisms found
    pub organisms: Vec<String>,
}

/// Natural genome match
#[derive(Debug, Clone, Serialize)]
pub struct NaturalMatch {
    /// Type of match
    #[serde(rename = "type")]
    pub match_type: String,

    /// Gene or organism name
    pub name: String,

    /// Matching signature
    pub signature: String,

    /// Position in sequence
    pub position: usize,
}

/// Sequence characteristics
#[derive(Debug, Clone, Serialize)]
pub struct SequenceCharacteristics {
    /// Sequence length
    pub length: usize,

    /// GC content percentage
    pub gc_content: f64,

    /// Homopolymer runs
    pub homopolymer_runs: Vec<HomopolymerRun>,

    /// Open reading frames
    pub orfs: Vec<OpenReadingFrame>,

    /// Repetitive elements
    pub repetitive_elements: Vec<RepetitiveElement>,

    /// Warnings
    pub warnings: Vec<String>,
}

/// Homopolymer run
#[derive(Debug, Clone, Serialize)]
pub struct HomopolymerRun {
    /// Base character
    pub base: char,

    /// Run length
    pub length: usize,

    /// Start position
    pub position: usize,
}

/// Open reading frame
#[derive(Debug, Clone, Serialize)]
pub struct OpenReadingFrame {
    /// Start position
    pub start: usize,

    /// End position
    pub end: usize,

    /// Reading frame
    pub frame: usize,
}

/// Repetitive element
#[derive(Debug, Clone, Serialize)]
pub struct RepetitiveElement {
    /// Pattern
    pub pattern: String,

    /// Count
    pub count: usize,

    /// Pattern length
    pub length: usize,
}

/// Sequence statistics (simplified version)
#[derive(Debug, Clone, Serialize)]
pub struct SequenceStats {
    /// Sequence length
    pub length: usize,

    /// Base counts
    pub bases: BaseCounts,

    /// GC content percentage
    pub gc_content: f64,
}

/// Base count statistics
#[derive(Debug, Clone, Serialize)]
pub struct BaseCounts {
    pub a: usize,
    pub t: usize,
    pub c: usize,
    pub g: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_status_icon() {
        assert_eq!(SafetyStatus::Safe.icon(), "✅");
        assert_eq!(SafetyStatus::Caution.icon(), "⚠️");
        assert_eq!(SafetyStatus::Unsafe.icon(), "❌");
    }

    #[test]
    fn test_safety_status_color() {
        assert_eq!(SafetyStatus::Safe.color(), "green");
        assert_eq!(SafetyStatus::Caution.color(), "orange");
        assert_eq!(SafetyStatus::Unsafe.color(), "red");
    }
}
