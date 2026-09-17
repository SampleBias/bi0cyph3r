//! Terminal workbench with scriptable subcommands.
use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
    process::ExitCode,
};

use anyhow::{bail, Context, Result};
use biocypher_backend::{
    dna::EncodingMode,
    workbench::{self, Format, Operation, Request},
};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bi0cyph3r",
    version,
    about = "A local DNA workbench. Run without arguments to open the Ratatui TUI."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Open the interactive terminal workbench.
    Tui,
    /// Encode UTF-8 text as DNA.
    Encode(OperationArgs),
    /// Decode DNA or one FASTA record.
    Decode(OperationArgs),
    /// Inspect a DNA sequence using the local safety screener.
    #[command(alias = "screen")]
    Safety(OperationArgs),
    /// Prepare a named DNA payload (FASTA by default).
    Plasmid(OperationArgs),
}

#[derive(Args)]
struct OperationArgs {
    /// Text or DNA; use "-" to read from stdin.
    #[arg(conflicts_with = "input")]
    text: Option<String>,
    /// Read a UTF-8 text or single-record FASTA file.
    #[arg(short, long)]
    input: Option<PathBuf>,
    #[arg(long, default_value = "basic")]
    mode: EncodingMode,
    /// Password for secure mode (prefer BIOCYPHER_PASSWORD over shell arguments).
    #[arg(short, long, env = "BIOCYPHER_PASSWORD", hide_env_values = true)]
    password: Option<String>,
    #[arg(long, env = "BIOCYPHER_K1", hide_env_values = true)]
    k1: Option<String>,
    #[arg(long, env = "BIOCYPHER_K2", hide_env_values = true)]
    k2: Option<String>,
    #[arg(long, default_value = "biocypher_payload")]
    name: String,
    /// Plasmid layout: payload only, decoding markers, or markers plus eGFP.
    #[arg(long, value_enum, default_value = "payload")]
    structure: biocypher_backend::plasmid::Structure,
    /// Result format; defaults to text, or FASTA for plasmid.
    #[arg(long, alias = "format", value_enum)]
    output: Option<Format>,
    /// Save to a new file instead of stdout; never overwrite existing files.
    #[arg(long)]
    save: Option<PathBuf>,
    /// Export both split keys to a new private JSON file; required for split-key encoding.
    #[arg(long)]
    keys_output: Option<PathBuf>,
    /// Record a Solana attestation using the configured local keypair.
    #[cfg(feature = "solana")]
    #[arg(long)]
    attest: bool,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error
                .downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
            {
                return ExitCode::SUCCESS;
            }
            eprintln!("Error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let (operation, args) = match cli.command {
        None | Some(Command::Tui) => return biocypher_backend::tui::run(),
        Some(Command::Encode(args)) => (Operation::Encode, args),
        Some(Command::Decode(args)) => (Operation::Decode, args),
        Some(Command::Safety(args)) => (Operation::Safety, args),
        Some(Command::Plasmid(args)) => (Operation::Plasmid, args),
    };
    let generates_keys = args.mode == EncodingMode::SplitKey
        && matches!(operation, Operation::Encode | Operation::Plasmid);
    if generates_keys && args.keys_output.is_none() {
        bail!("Split-key encoding requires --keys-output <new-file.json> so neither key is lost");
    }
    if !generates_keys && args.keys_output.is_some() {
        bail!("--keys-output is only used when encoding a message in splitkey mode");
    }
    let input = if let Some(path) = args.input {
        workbench::read_input(&path)?
    } else if args.text.as_deref() == Some("-")
        || (args.text.is_none() && !io::stdin().is_terminal())
    {
        workbench::read_limited(io::stdin().lock())?
    } else {
        args.text
            .context("Provide text, --input FILE, or pipe text through stdin")?
    };
    let result = workbench::execute(&Request {
        operation,
        input,
        mode: args.mode,
        password: args.password.unwrap_or_default(),
        k1: args.k1.unwrap_or_default(),
        k2: args.k2.unwrap_or_default(),
        name: args.name,
        structure: args.structure,
    })?;
    if let Some(keys) = &result.keys {
        let path = args.keys_output.as_ref().unwrap();
        workbench::save_new(path, &serde_json::to_string_pretty(keys)?)?;
        eprintln!(
            "Keys saved to {}. Store K1 and K2 separately.",
            path.display()
        );
    }
    let format = args.output.unwrap_or(if operation == Operation::Plasmid {
        Format::Fasta
    } else {
        Format::Txt
    });
    let contents = result.export(format)?;
    if let Some(path) = &args.save {
        workbench::save_new(path, &contents)?;
        eprintln!("Saved {}", path.display());
    } else {
        io::stdout().lock().write_all(contents.as_bytes())?;
    }
    eprintln!(
        "[{} bases · GC {:.1}% · {}]",
        result.stats.length, result.stats.gc_content, result.mode
    );
    #[cfg(feature = "solana")]
    if args.attest {
        use biocypher_backend::solana::{hash_sequence, SolanaClient};
        let client =
            SolanaClient::from_env().context("Solana keypair or program ID is not configured")?;
        let hash = hash_sequence(&result.sequence);
        let runtime = tokio::runtime::Runtime::new()?;
        let signature = runtime.block_on(async {
            match operation {
                Operation::Encode | Operation::Plasmid => {
                    client.record_encode(result.mode, hash).await
                }
                Operation::Decode => client.record_decode(result.mode, hash).await,
                Operation::Safety => {
                    client
                        .record_safety(hash, result.report.as_ref().unwrap().safety_status)
                        .await
                }
            }
        })?;
        eprintln!("Solana attestation: {signature}");
    }
    Ok(())
}
