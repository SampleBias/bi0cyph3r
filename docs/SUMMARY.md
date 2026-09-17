# Project summary

Bi0cyph3r is a local DNA workbench with a Ratatui terminal interface and scriptable Rust CLI.

It supports Basic, Nanopore, Secure, and Split Key encoding, DNA / FASTA decoding, heuristic sequence screening, and plasmid payload assembly with optional decoding markers and the existing eGFP cassette. The interface includes colored bases, GC statistics, masked secrets, background processing, file import/export, and keyboard help.

The application no longer includes an HTTP server, browser frontend, Flask app, or GUI assets. Solana attestations remain opt-in via the CLI's optional `solana` feature.

See the [README](../README.md) for current instructions.
