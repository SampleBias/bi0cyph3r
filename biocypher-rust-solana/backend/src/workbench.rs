//! Shared, local operations for the TUI and scriptable CLI.

use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::{
    dna::{
        DNACoder, DNACrypto, EncodingMode, NanoporeDNACrypto, SecureDNACrypto, SequenceStatistics,
        SplitKeyDNACrypto,
    },
    safety::{screener::SafetyReport, DNASafetyScreener},
};

pub const MAX_INPUT_BYTES: usize = 256 * 1024;
// Keeps even the largest Nanopore output within the sequence import limit.
pub const MAX_MESSAGE_BYTES: usize = 4 * 1024;
pub const MODES: [EncodingMode; 4] = [
    EncodingMode::Basic,
    EncodingMode::Nanopore,
    EncodingMode::Secure,
    EncodingMode::SplitKey,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Operation {
    Encode,
    Decode,
    Safety,
    Plasmid,
}

impl Operation {
    pub const ALL: [Self; 4] = [Self::Encode, Self::Decode, Self::Safety, Self::Plasmid];
    pub fn title(self) -> &'static str {
        match self {
            Self::Encode => "Encode",
            Self::Decode => "Decode",
            Self::Safety => "Safety",
            Self::Plasmid => "Plasmid",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Encode => "Turn a message into a strand of DNA.",
            Self::Decode => "Recover a message from a DNA sequence or FASTA file.",
            Self::Safety => "Inspect sequence characteristics and local signature matches.",
            Self::Plasmid => "Prepare a named DNA payload for FASTA export.",
        }
    }
}

#[derive(Clone)]
pub struct Request {
    pub operation: Operation,
    pub input: String,
    pub mode: EncodingMode,
    pub password: String,
    pub k1: String,
    pub k2: String,
    pub name: String,
    pub structure: crate::plasmid::Structure,
}

#[derive(Clone, Serialize)]
pub struct Keys {
    pub k1_base64: String,
    pub k2_base64: String,
}

#[derive(Clone, Serialize)]
pub struct Output {
    pub operation: Operation,
    pub mode: EncodingMode,
    pub name: String,
    pub sequence: String,
    pub text: String,
    pub stats: SequenceStatistics,
    pub features: Vec<crate::plasmid::Feature>,
    pub report: Option<SafetyReport>,
    // Secret material is only written by an explicit key export.
    #[serde(skip)]
    pub keys: Option<Keys>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Txt,
    Fasta,
    Json,
}

impl Format {
    pub fn next(self) -> Self {
        match self {
            Self::Txt => Self::Fasta,
            Self::Fasta => Self::Json,
            Self::Json => Self::Txt,
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Fasta => "fasta",
            Self::Json => "json",
        }
    }
}

impl Output {
    pub fn export(&self, format: Format) -> Result<String> {
        Ok(match format {
            Format::Txt if self.operation == Operation::Decode => self.text.clone(),
            Format::Txt => format!("{}\n", self.text),
            Format::Fasta => {
                let mut fasta = format!(">{} mode={}\n", self.name, self.mode);
                for chunk in self.sequence.as_bytes().chunks(80) {
                    fasta.push_str(std::str::from_utf8(chunk)?);
                    fasta.push('\n');
                }
                fasta
            }
            Format::Json => format!("{}\n", serde_json::to_string_pretty(self)?),
        })
    }
}

/// Accept one FASTA record or raw DNA. Reject invalid bases rather than silently
/// discarding them, and never concatenate unrelated FASTA records.
pub fn normalize_sequence(input: &str) -> Result<String> {
    let mut sequence = String::new();
    let mut headers = 0;
    for line in input.lines() {
        let line = line.trim();
        if line.starts_with('>') {
            headers += 1;
            if headers > 1 || !sequence.is_empty() {
                bail!("Import a single FASTA record at a time");
            }
            continue;
        }
        for ch in line.chars().filter(|c| !c.is_whitespace()) {
            let base = ch.to_ascii_uppercase();
            if !matches!(base, 'A' | 'T' | 'C' | 'G') {
                bail!("Invalid DNA base {ch:?}; expected A, T, C or G");
            }
            sequence.push(base);
        }
    }
    if sequence.is_empty() {
        bail!("Enter a DNA sequence first");
    }
    Ok(sequence)
}

pub fn execute(request: &Request) -> Result<Output> {
    if request.input.is_empty() {
        bail!("Enter a message or import a file first");
    }
    if request.input.len() > MAX_INPUT_BYTES {
        bail!("Input exceeds the 256 KiB limit");
    }
    if matches!(request.operation, Operation::Encode | Operation::Plasmid)
        && request.input.len() > MAX_MESSAGE_BYTES
    {
        bail!("Messages are limited to 4 KiB so generated DNA fits the 256 KiB sequence limit");
    }
    let mut keys = None;
    let mut report = None;
    let mut features = Vec::new();
    let (sequence, text) = match request.operation {
        Operation::Encode | Operation::Plasmid => {
            let sequence = match request.mode {
                EncodingMode::Basic => DNACrypto::encode_message(&request.input)?,
                EncodingMode::Nanopore => NanoporeDNACrypto::encode_message(&request.input)?,
                EncodingMode::Secure => {
                    SecureDNACrypto::encode_with_password(&request.input, &request.password)?
                }
                EncodingMode::SplitKey => {
                    let (seq, k1_base64, k2_base64) =
                        SplitKeyDNACrypto::encode_with_split_keys(&request.input)?;
                    keys = Some(Keys {
                        k1_base64,
                        k2_base64,
                    });
                    seq
                }
            };
            let sequence = if request.operation == Operation::Plasmid {
                let (assembled, annotations) =
                    crate::plasmid::assemble(&sequence, request.structure);
                features = annotations;
                assembled
            } else {
                sequence
            };
            (sequence.clone(), sequence)
        }
        Operation::Decode => {
            let sequence = normalize_sequence(&request.input)?;
            let sequence = crate::plasmid::payload(&sequence)?.to_owned();
            if request.mode == EncodingMode::Basic && sequence.len() % 4 != 0 {
                bail!("Basic DNA must contain a multiple of 4 bases (one UTF-8 byte per 4 bases)");
            }
            let text = match request.mode {
                EncodingMode::Basic => DNACrypto::decode_sequence(&sequence)?,
                EncodingMode::Nanopore => NanoporeDNACrypto::decode_sequence(&sequence)?,
                EncodingMode::Secure => {
                    SecureDNACrypto::decode_with_password(&sequence, &request.password)?
                }
                EncodingMode::SplitKey => SplitKeyDNACrypto::decode_with_split_keys(
                    &sequence,
                    request.k1.trim(),
                    request.k2.trim(),
                )?,
            };
            (sequence, text)
        }
        Operation::Safety => {
            let sequence = normalize_sequence(&request.input)?;
            let result = DNASafetyScreener::new().perform_comprehensive_screening(&sequence)?;
            let mut text = format!(
                "SCREENING RESULT  /  {:?}\n\n{} bases   ·   GC {:.1}%\n\nSignature matches    {}\nNatural matches      {}\nHomopolymer runs     {}\nOpen reading frames  {}\n\n",
                result.safety_status, sequence.len(), result.sequence_characteristics.gc_content,
                result.pathogen_analysis.matches.len(), result.natural_occurrence.matches.len(),
                result.sequence_characteristics.homopolymer_runs.len(), result.sequence_characteristics.orfs.len());
            for recommendation in &result.recommendations {
                text.push_str(&format!("• {recommendation}\n"));
            }
            text.push_str(
                "\nLocal heuristic screening only; this is not a biological safety certification.",
            );
            report = Some(result);
            (sequence, text)
        }
    };
    let name = request.name.trim();
    if name.len() > 128 || name.chars().any(char::is_control) {
        bail!("Sequence name must be at most 128 bytes with no control characters");
    }
    Ok(Output {
        operation: request.operation,
        mode: request.mode,
        name: if name.is_empty() {
            "biocypher_payload".into()
        } else {
            name.into()
        },
        stats: SequenceStatistics::new(&sequence),
        features,
        sequence,
        text,
        keys,
        report,
    })
}

pub fn read_input(path: &Path) -> Result<String> {
    let file =
        std::fs::File::open(path).with_context(|| format!("Cannot open {}", path.display()))?;
    read_limited(file)
}

pub fn read_limited(reader: impl Read) -> Result<String> {
    let mut input = String::new();
    reader
        .take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_string(&mut input)
        .context("Input must be UTF-8 text")?;
    if input.len() > MAX_INPUT_BYTES {
        bail!("Input exceeds the 256 KiB limit");
    }
    Ok(input)
}

/// Create only: never truncate an existing file, including a symlink target.
pub fn save_new(path: &Path, contents: &str) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).with_context(|| {
        format!(
            "Cannot create {} (choose a new path if it exists)",
            path.display()
        )
    })?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(mode: EncodingMode) -> Request {
        Request {
            operation: Operation::Encode,
            input: "Hello, DNA!".into(),
            mode,
            password: "test-password".into(),
            k1: String::new(),
            k2: String::new(),
            name: "test".into(),
            structure: crate::plasmid::Structure::Payload,
        }
    }

    #[test]
    fn all_modes_roundtrip_through_fasta() {
        for mode in MODES {
            let mut req = request(mode);
            let encoded = execute(&req).unwrap();
            req.input = encoded.export(Format::Fasta).unwrap();
            req.operation = Operation::Decode;
            if let Some(keys) = encoded.keys {
                req.k1 = keys.k1_base64;
                req.k2 = keys.k2_base64;
            }
            assert_eq!(execute(&req).unwrap().text, "Hello, DNA!");
        }
    }

    #[test]
    fn utf8_and_newlines_roundtrip_without_extra_bytes() {
        for mode in MODES {
            let mut req = request(mode);
            req.input = "Hello 🧬\nCafé\t世界\n".into();
            let encoded = execute(&req).unwrap();
            req.input = encoded.sequence;
            req.operation = Operation::Decode;
            if let Some(keys) = encoded.keys {
                req.k1 = keys.k1_base64;
                req.k2 = keys.k2_base64;
            }
            assert_eq!(
                execute(&req).unwrap().export(Format::Txt).unwrap(),
                "Hello 🧬\nCafé\t世界\n"
            );
        }
    }

    #[test]
    fn plasmid_structures_preserve_payload_and_export_annotations() {
        use crate::plasmid::Structure;
        for structure in [
            Structure::Payload,
            Structure::Marked,
            Structure::Fluorescent,
        ] {
            let mut req = request(EncodingMode::Basic);
            req.operation = Operation::Plasmid;
            req.structure = structure;
            let output = execute(&req).unwrap();
            assert_eq!(output.features.first().unwrap().start, 0);
            assert_eq!(output.features.last().unwrap().end, output.sequence.len());
            req.operation = Operation::Decode;
            req.input = output.export(Format::Fasta).unwrap();
            assert_eq!(execute(&req).unwrap().text, "Hello, DNA!");
        }
    }

    #[test]
    fn largest_nanopore_plasmid_remains_importable() {
        let mut req = request(EncodingMode::Nanopore);
        req.input = "A".repeat(MAX_MESSAGE_BYTES);
        req.operation = Operation::Plasmid;
        req.structure = crate::plasmid::Structure::Fluorescent;
        let output = execute(&req).unwrap();
        let fasta = output.export(Format::Fasta).unwrap();
        assert!(fasta.len() <= MAX_INPUT_BYTES);
        req.operation = Operation::Decode;
        req.input = fasta;
        assert_eq!(execute(&req).unwrap().text, "A".repeat(MAX_MESSAGE_BYTES));
    }

    #[test]
    fn fasta_validation_does_not_silently_corrupt_input() {
        assert_eq!(normalize_sequence(">sample\nat cg\nTA").unwrap(), "ATCGTA");
        for input in [
            "",
            ">header",
            "ATNCG",
            ">a\nATCG\n>b\nATCG",
            "ATCG\n>b\nATCG",
        ] {
            assert!(normalize_sequence(input).is_err(), "{input}");
        }
    }

    #[test]
    fn exports_do_not_leak_split_keys() {
        let output = execute(&request(EncodingMode::SplitKey)).unwrap();
        let keys = output.keys.as_ref().unwrap();
        for format in [Format::Txt, Format::Fasta, Format::Json] {
            let text = output.export(format).unwrap();
            assert!(!text.contains(&keys.k1_base64));
            assert!(!text.contains(&keys.k2_base64));
        }
    }

    #[test]
    fn save_refuses_overwrite() {
        let path = std::env::temp_dir().join(format!(
            "biocypher-test-{}-{}.txt",
            std::process::id(),
            rand::random::<u64>()
        ));
        save_new(&path, "original").unwrap();
        assert!(save_new(&path, "replacement").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        std::fs::remove_file(path).unwrap();
    }
}
