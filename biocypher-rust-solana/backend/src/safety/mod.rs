//! Safety screening module
//!
//! Analyzes DNA sequences for potential pathogen risks and natural occurrence
//! Native Rust port of the original BioCypher screener (see docs/LEGACY_LICENSE).

pub mod screener;

pub use screener::DNASafetyScreener;
