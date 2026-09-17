//! DNA Safety Screener Module
//!
//! Analyzes DNA sequences for potential pathogen risks and natural occurrence
//! Native Rust port of the original BioCypher screener (see docs/LEGACY_LICENSE).

use crate::dna::traits::SequenceStatistics;
use crate::error::{Result, SafetyScreenerError as Error};
use crate::models::*;

/// DNA Safety Screening System
#[derive(Default)]
pub struct DNASafetyScreener;

impl DNASafetyScreener {
    /// Create new safety screener instance
    pub fn new() -> Self {
        Self
    }

    /// Perform comprehensive safety screening
    pub fn perform_comprehensive_screening(&self, sequence: &str) -> Result<SafetyReport> {
        // Clean sequence
        let clean_sequence = self.clean_sequence(sequence)?;

        // Perform all checks
        let pathogen_check = self.check_pathogen_signatures(&clean_sequence);
        let natural_check = self.check_natural_occurrence(&clean_sequence);
        let characteristics = self.analyze_characteristics(&clean_sequence);

        // Determine overall safety status
        let safety_status =
            self.determine_status(&pathogen_check, &natural_check, &characteristics);

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &pathogen_check,
            &natural_check,
            &characteristics,
            safety_status,
        );

        Ok(SafetyReport {
            dna_sequence: clean_sequence,
            safety_status,
            pathogen_analysis: pathogen_check,
            natural_occurrence: natural_check,
            sequence_characteristics: characteristics,
            recommendations,
        })
    }

    /// Clean DNA sequence (remove non-ATCG characters)
    fn clean_sequence(&self, sequence: &str) -> Result<String> {
        let cleaned: String = sequence
            .chars()
            .filter(|c| matches!(c.to_ascii_uppercase(), 'A' | 'T' | 'C' | 'G'))
            .collect();

        if cleaned.is_empty() && !sequence.is_empty() {
            return Err(Error::NoValidBases.into());
        }

        Ok(cleaned)
    }

    /// Check for known pathogen signatures
    fn check_pathogen_signatures(&self, sequence: &str) -> PathogenAnalysis {
        // Simplified pathogen signatures
        let signatures = vec![
            (
                "viral_polymerase",
                vec!["ATGGATCCGTATGACTCC", "CCGTATGACTCCATGG"],
            ),
            (
                "toxin_genes",
                vec!["ATGAAGCTGTATGACCC", "GGGTCATACAGCTTCAT"],
            ),
            (
                "antibiotic_resistance",
                vec![
                    "ATGAGCCATATTCAACG",
                    "CGTTGAATATGGCTCAT",
                    "ATGTCGCAGTTCGATCC",
                    "GGATCGAACTGCGACAT",
                ],
            ),
            (
                "virulence_factors",
                vec!["ATGCTGAAACGTTATGC", "GCATAACGTTTCAGCAT"],
            ),
        ];

        let mut matches = Vec::new();
        let sequence_upper = sequence.to_uppercase();

        for (category, sigs) in signatures {
            for signature in sigs {
                if let Some(pos) = sequence_upper.find(signature) {
                    matches.push(PathogenMatch {
                        category: category.to_string(),
                        signature: signature.to_string(),
                        position: pos,
                    });
                }
            }
        }

        let (pathogen_risk, risk_level) = if !matches.is_empty() {
            if matches
                .iter()
                .any(|m| m.category == "toxin_genes" || m.category == "virulence_factors")
            {
                (true, RiskLevel::High)
            } else if matches
                .iter()
                .any(|m| m.category == "antibiotic_resistance")
            {
                (true, RiskLevel::Medium)
            } else {
                (true, RiskLevel::Low)
            }
        } else {
            (false, RiskLevel::Low)
        };

        PathogenAnalysis {
            pathogen_risk,
            matches,
            risk_level,
        }
    }

    /// Check for natural genome occurrences
    fn check_natural_occurrence(&self, sequence: &str) -> NaturalOccurrence {
        // Simplified natural genome signatures
        let housekeeping = vec![
            ("ribosomal_rna", "TACCTGGTTGATCCTGC"),
            ("actin", "ATGGATGATGATATCGC"),
            ("gapdh", "ATGGTGAAGGTCGGTGT"),
            ("tubulin", "ATGCGTGAGATCGTGCA"),
        ];

        let ecoli = vec![
            "GATCCTGGAAAGTGCAG",
            "CTGCACTTTCCAGGATC",
            "ATGAAACGCATTAGCAC",
            "GTGCTAATGCGTTTCAT",
        ];

        let human = vec![
            "ATGCCCTGTGATTTCGG",
            "CCGAAATCACAGGGCAT",
            "GAGCTGAAGGGCGTGAA",
            "TTCACGCCCTTCAGCTC",
        ];

        let mut matches = Vec::new();
        let mut organisms = Vec::new();
        let sequence_upper = sequence.to_uppercase();

        // Check housekeeping genes
        for (gene, signature) in housekeeping {
            if let Some(pos) = sequence_upper.find(signature) {
                matches.push(NaturalMatch {
                    match_type: "housekeeping_gene".to_string(),
                    name: gene.to_string(),
                    signature: signature.to_string(),
                    position: pos,
                });
                if !organisms.contains(&"Multiple species (housekeeping)".to_string()) {
                    organisms.push("Multiple species (housekeeping)".to_string());
                }
            }
        }

        // Check E. coli
        for signature in ecoli {
            if let Some(pos) = sequence_upper.find(signature) {
                matches.push(NaturalMatch {
                    match_type: "genome_signature".to_string(),
                    name: "E. coli".to_string(),
                    signature: signature.to_string(),
                    position: pos,
                });
                if !organisms.contains(&"E. coli".to_string()) {
                    organisms.push("E. coli".to_string());
                }
            }
        }

        // Check human
        for signature in human {
            if let Some(pos) = sequence_upper.find(signature) {
                matches.push(NaturalMatch {
                    match_type: "genome_signature".to_string(),
                    name: "Human".to_string(),
                    signature: signature.to_string(),
                    position: pos,
                });
                if !organisms.contains(&"Human".to_string()) {
                    organisms.push("Human".to_string());
                }
            }
        }

        let natural_occurrence = !matches.is_empty();

        NaturalOccurrence {
            natural_occurrence,
            matches,
            organisms,
        }
    }

    /// Analyze sequence characteristics
    fn analyze_characteristics(&self, sequence: &str) -> SequenceCharacteristics {
        let stats = SequenceStatistics::new(sequence);

        let mut homopolymer_runs = Vec::new();
        let mut orfs = Vec::new();
        let mut repetitive_elements = Vec::new();
        let mut warnings = Vec::new();

        // Check for extreme GC content
        if stats.gc_content < 20.0 || stats.gc_content > 80.0 {
            warnings.push(format!("Extreme GC content: {:.1}%", stats.gc_content));
        }

        // Find homopolymer runs (4+ consecutive same bases)
        for base in ['A', 'T', 'C', 'G'] {
            let pattern = format!("{}{{{{4,}}}}", base);
            if let Ok(re) = regex::Regex::new(&pattern) {
                for mat in re.find_iter(sequence) {
                    homopolymer_runs.push(HomopolymerRun {
                        base,
                        length: mat.as_str().len(),
                        position: mat.start(),
                    });
                }
            }
        }

        // Warn about long homopolymer runs
        for run in &homopolymer_runs {
            if run.length >= 6 {
                warnings.push(format!(
                    "Long homopolymer run: {} {}s",
                    run.length, run.base
                ));
            }
        }

        // Find ORFs (simplified)
        let start_codons = ["ATG"];
        let stop_codons = ["TAA", "TAG", "TGA"];

        for frame in 0..3 {
            let mut i = frame;
            while i + 2 < sequence.len() {
                let codon = &sequence[i..i + 3];
                if start_codons.contains(&codon) {
                    for j in (i + 3..sequence.len() - 2).step_by(3) {
                        let stop_codon = &sequence[j..j + 3];
                        if stop_codons.contains(&stop_codon) {
                            if j - i >= 30 {
                                orfs.push(OpenReadingFrame {
                                    start: i,
                                    end: j + 3,
                                    frame,
                                });
                            }
                            break;
                        }
                    }
                }
                i += 3;
            }
        }

        if !orfs.is_empty() {
            warnings.push(format!(
                "Found {} potential protein-coding sequences",
                orfs.len()
            ));
        }

        // Check for repetitive elements (simplified)
        for i in 3..std::cmp::min(20, sequence.len() / 3) {
            let pattern = &sequence[..i];
            let count = sequence.matches(pattern).count();
            if count >= 3 {
                repetitive_elements.push(RepetitiveElement {
                    pattern: pattern.to_string(),
                    count,
                    length: i,
                });
            }
        }

        if !repetitive_elements.is_empty() {
            warnings.push("Repetitive elements detected".to_string());
        }

        SequenceCharacteristics {
            length: stats.length,
            gc_content: stats.gc_content,
            homopolymer_runs,
            orfs,
            repetitive_elements,
            warnings,
        }
    }

    /// Determine overall safety status
    fn determine_status(
        &self,
        pathogen_check: &PathogenAnalysis,
        natural_check: &NaturalOccurrence,
        characteristics: &SequenceCharacteristics,
    ) -> SafetyStatus {
        if pathogen_check.pathogen_risk {
            return match pathogen_check.risk_level {
                RiskLevel::High => SafetyStatus::Unsafe,
                RiskLevel::Medium | RiskLevel::Low => SafetyStatus::Caution,
            };
        }

        if natural_check.natural_occurrence {
            return SafetyStatus::Caution;
        }

        if characteristics.warnings.len() > 2 {
            return SafetyStatus::Caution;
        }

        SafetyStatus::Safe
    }

    /// Generate safety recommendations
    fn generate_recommendations(
        &self,
        pathogen_check: &PathogenAnalysis,
        natural_check: &NaturalOccurrence,
        characteristics: &SequenceCharacteristics,
        safety_status: SafetyStatus,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if matches!(safety_status, SafetyStatus::Safe) {
            recommendations.push("✅ Sequence appears safe for synthesis".to_string());
            recommendations.push("✅ No pathogen signatures detected".to_string());
            recommendations.push("✅ No natural genome matches found".to_string());
        }

        if pathogen_check.pathogen_risk {
            recommendations.push("❌ DO NOT SYNTHESIZE - Pathogen signatures detected".to_string());
            recommendations.push("🔬 Consult biosafety experts before proceeding".to_string());
        }

        if natural_check.natural_occurrence {
            recommendations.push("⚠️ Sequence matches natural genomes".to_string());
            recommendations.push("🧬 Consider modifying to avoid natural sequences".to_string());
        }

        if characteristics.gc_content < 20.0 || characteristics.gc_content > 80.0 {
            recommendations.push(format!(
                "⚠️ Extreme GC content ({:.1}%) may affect synthesis",
                characteristics.gc_content
            ));
        }

        if !characteristics.orfs.is_empty() {
            recommendations.push("🧬 Contains potential protein-coding sequences".to_string());
        }

        if characteristics.homopolymer_runs.len() > 5 {
            recommendations.push("⚠️ Multiple homopolymer runs detected".to_string());
        }

        recommendations
    }
}

/// Internal safety report
#[derive(Clone, serde::Serialize)]
pub struct SafetyReport {
    pub dna_sequence: String,
    pub safety_status: SafetyStatus,
    pub pathogen_analysis: PathogenAnalysis,
    pub natural_occurrence: NaturalOccurrence,
    pub sequence_characteristics: SequenceCharacteristics,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_sequence() {
        let screener = DNASafetyScreener::new();
        let cleaned = screener.clean_sequence("ATCGX YZNATCG").unwrap();
        // Only A,T,C,G are kept; X, space, Y, Z, N are filtered out
        assert_eq!(cleaned, "ATCGATCG");
    }

    #[test]
    fn test_clean_sequence_no_valid_bases() {
        let screener = DNASafetyScreener::new();
        let result = screener.clean_sequence("XYZ123");
        assert!(result.is_err());
    }

    #[test]
    fn test_comprehensive_screening() {
        let screener = DNASafetyScreener::new();
        let sequence = "ATCGATCGATCGATCG";
        let report = screener.perform_comprehensive_screening(sequence);
        assert!(report.is_ok());
    }

    #[test]
    fn test_pathogen_detection() {
        let screener = DNASafetyScreener::new();
        let analysis = screener.check_pathogen_signatures("ATGGATCCGTATGACTCC");
        assert!(analysis.pathogen_risk);
        assert!(!analysis.matches.is_empty());
    }

    #[test]
    fn test_no_pathogen_risk() {
        let screener = DNASafetyScreener::new();
        let analysis = screener.check_pathogen_signatures("ATCGATCG");
        assert!(!analysis.pathogen_risk);
    }

    #[test]
    fn test_natural_occurrence() {
        let screener = DNASafetyScreener::new();
        let check = screener.check_natural_occurrence("ATGGATGATGATATCGC");
        assert!(check.natural_occurrence);
    }
}
