//! Plasmid assembly carried over from the former browser designer.
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Structure {
    #[default]
    Payload,
    Marked,
    Fluorescent,
}

impl Structure {
    pub fn next(self) -> Self {
        match self {
            Self::Payload => Self::Marked,
            Self::Marked => Self::Fluorescent,
            Self::Fluorescent => Self::Payload,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Payload => "Payload only",
            Self::Marked => "Decoding markers",
            Self::Fluorescent => "Markers + eGFP",
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Feature {
    pub name: String,
    /// Zero-based, half-open coordinates.
    pub start: usize,
    pub end: usize,
}

const MARKER_START: &str = "CCTAGGGATCCATGCCTAG";
const MARKER_END: &str = "CCTAGGCATGGATCCCTAGG";
const PROMOTER: &str = "TAATACGACTCACTATAGGGAGA";
const RBS: &str = "AGGAGGA";
const TERMINATOR: &str = "CTAGCATAACCCCTTGGGGCCTCTAAACGGGTCTTGAGGGGTTTTTTG";
const EGFP: &str = "ATGCGTAAAGGAGAAGAACTTTTCACTGGAGTTGTCCCAATTCTTGTTGAATTAGATGGTGATGTTAATGGGCACAAATTTTCTGTCAGTGGAGAGGGTGAAGGTGATGCAACATACGGAAAACTTACCCTTAAATTTATTTGCACTACTGGAAAACTACCTGTTCCATGGCCAACACTTGTCACTACTTTCGGTTATGGTGTTCAATGCTTTGCGAGATACCCAGATCATATGAAACAGCATGACTTTTTCAAGAGTGCCATGCCCGAAGGTTATGTACAGGAAAGAACTATATTTTTCAAAGATGACGGGAACTACAAGACACGTGCTGAAGTCAAGTTTGAAGGTGATACCCTTGTTAATAGAATCGAGTTAAAAGGTATTGATTTTAAAGAAGATGGAAACATTCTTGGACACAAATTGGAATACAACTATAACTCACACAATGTATACATCATGGCAGACAAACAAAAGAATGGAATCAAAGTTAACTTCAAAATTAGACACAACATTGAAGATGGAAGCGTTCAACTAGCAGACCATTATCAACAAAATACTCCAATTGGCGATGGCCCTGTCCTTTTACCAGACAACCATTACCTGTCCACACAATCTGCCCTTTCGAAAGATCCCAACGAAAAGAGAGACCACATGGTCCTTCTTGAGTTTGTAACAGCTGCTGGGATTACACATGGCATGGATGAACTATACAAATAATAA";

pub fn assemble(payload: &str, structure: Structure) -> (String, Vec<Feature>) {
    let mut sequence = String::new();
    let mut features = Vec::new();
    let mut append = |name: &str, bases: &str| {
        let start = sequence.len();
        sequence.push_str(bases);
        features.push(Feature {
            name: name.into(),
            start,
            end: sequence.len(),
        });
    };
    if structure != Structure::Payload {
        append("Marker start", MARKER_START);
    }
    append("Payload", payload);
    if structure != Structure::Payload {
        append("Marker end", MARKER_END);
    }
    if structure == Structure::Fluorescent {
        append("Promoter", PROMOTER);
        append("RBS", RBS);
        append("eGFP", EGFP);
        append("Terminator", TERMINATOR);
    }
    (sequence, features)
}

/// Extract the payload from the existing marked plasmid format.
pub fn payload(sequence: &str) -> anyhow::Result<&str> {
    if let Some(rest) = sequence.strip_prefix(MARKER_START) {
        let (payload, _) = rest
            .split_once(MARKER_END)
            .ok_or_else(|| anyhow::anyhow!("Plasmid start marker found without an end marker"))?;
        Ok(payload)
    } else {
        Ok(sequence)
    }
}
