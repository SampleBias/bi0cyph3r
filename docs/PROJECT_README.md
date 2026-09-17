# Bi0cyph3r architecture

Bi0cyph3r is a terminal-only Rust application. Run `./run-tui.sh` or `bi0cyph3r`.

- Ratatui and Crossterm render the keyboard-driven interface.
- The TUI and Clap subcommands share local operations in `backend/src/workbench.rs`.
- DNA codecs, plasmid assembly, and the safety screener run in-process.
- TUI calculations run on a worker thread so terminal events remain responsive.
- Local file imports and explicit TXT / FASTA / JSON exports replace browser uploads and downloads.
- Split keys use a separate export, and existing files are never overwritten.
- Solana support is an optional Cargo feature used through explicit CLI attestations.

The HTTP server, static frontend, Flask application, browser wallet integration, server escrow, and manufacturer forwarding endpoints have been removed. There is no GUI build or web deployment.

For current installation and usage, see [README](../README.md). Older task lists and activity logs describe the historical web architecture.
