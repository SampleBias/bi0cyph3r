//! Bi0cyph3r — DNA cryptography library
//!
//! Encode and decode messages as DNA sequences. Supports Basic, Nanopore, and Secure modes.

pub mod dna;
pub mod error;
pub mod models;
pub mod plasmid;
pub mod safety;
#[cfg(feature = "solana")]
pub mod solana;
pub mod tui;
pub mod workbench;
