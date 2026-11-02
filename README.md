# Bi0cyph3r

Bi0cyph3r is a local, keyboard-driven DNA workbench for the terminal. It encodes and decodes UTF-8 text, inspects DNA sequences, and prepares plasmid payloads. The application runs in Rust without a server, browser, or blockchain connection.

The [open protocol specification](docs/PROTOCOL_SPECIFICATION.md) describes the formats and interoperability vectors. The original attribution is preserved in [docs/LEGACY_LICENSE](docs/LEGACY_LICENSE).

## Launch

Install stable Rust 1.88 or later. From the repository root, run:

```sh
./run-tui.sh
```

The first build may need network access to download Rust dependencies. Once built, local app operations do not require a network connection. A terminal at least 64 × 22 is recommended.

To build an optimized binary or install it for future use:

```sh
./install.sh        # Build biocypher-tui/target/release/bi0cyph3r
./install.sh -i     # Build and install to ~/.local/bin/bi0cyph3r
bi0cyph3r           # Launch after installing, with ~/.local/bin on PATH
```

On the start screen, press **s** to start, **g** for the guide, or **t** for themes. Arrow keys and Enter also navigate. Space pauses the helix animation. Set a launch theme with `bi0cyph3r --theme paper` or `BIOCYPHER_THEME=paper bi0cyph3r`. Themes are `original`, `crimson`, `paper`, `monochrome`, and `amber`.

## First encode and decode

1. Press **s** on the start screen to open the Encode workspace.
2. Press **i**, type a message, and press **Ctrl+R** to encode it using the default Basic mode.
3. Press **d** to send the resulting sequence to Decode, then **Ctrl+R** to recover the message.
4. Press **Ctrl+S** to export a result, or **q** to quit. Basic mode is an encoding and does not encrypt the message.

For password-protected data, press **m** until `secure` is selected, then use **Tab** to reach the password field. Enter a password before running the operation.

## Workbench

| Workspace | Purpose |
| --- | --- |
| Encode | Encode UTF-8 text as DNA |
| Decode | Decode raw DNA, one FASTA record, or a marked plasmid payload |
| Safety | Inspect sequence characteristics and local signature matches |
| Plasmid | Assemble a payload with optional markers or the existing eGFP cassette |

| Key | Action |
| --- | --- |
| `1`–`4` / `F1`–`F4` | Select a workspace |
| `Tab` / `Shift+Tab` | Move between fields |
| `i` / `Enter` | Edit the focused field |
| `Esc` | Finish editing, close a dialog, or return to the start screen |
| `m` | Cycle encoding mode |
| `x` | Cycle plasmid structure |
| `Ctrl+R` / `F5` | Run an operation |
| `Ctrl+O` | Import a UTF-8 text or single-record FASTA file |
| `Ctrl+S` / `F6` | Export TXT, FASTA, or JSON |
| `Ctrl+K` / `v` | Export / reveal generated split keys |
| `d` / `s` | Send the result to Decode / Safety |
| `t` / `F9` | Choose a theme |
| `g` / `?` | Open the guide / keyboard help |
| `q` / `Ctrl+Q` / `F10` | Quit |

## Encoding modes

- **Basic** maps UTF-8 bytes directly to DNA. It does not encrypt or authenticate data.
- **Nanopore** retains the historical triplet, parity, repetition, and padding format. Its screening and padding are heuristics; they do not guarantee synthesis or sequencing performance.
- **Secure** uses the version 2 authenticated format with a password, Argon2id, and AES-256-GCM.
- **Split Key** uses the same authenticated cipher with a random key split into two shares. Both shares are required to decode. Export and store them separately.
- **Legacy Secure** and **Legacy Split** decode historical AES-256-CBC sequences. These formats cannot authenticate data and should only be used to recover existing sequences. To migrate, decode with the matching legacy mode and encode the recovered text with the corresponding current mode.

Select the matching mode when decoding. Version 2 and legacy encryption use separate modes so a damaged new sequence cannot silently fall back to legacy decryption.

Imported sequences accept lowercase bases and whitespace but reject invalid bases and multiple FASTA records. Messages are limited to 4 KiB and sequence imports to 256 KiB. Exported files are created without overwriting existing paths, with owner-only permissions on Unix. Session data is held in memory and disappears on exit unless exported. Ordinary exports omit split keys; use the explicit key export.

The Safety workspace provides local heuristic screening, not a biological safety certification. Plasmid assembly does not validate a complete expression-ready vector. No automatic manufacturer delivery or key escrow is performed.

## Development

```sh
cd biocypher-tui
cargo fmt --all --check
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The Rust workspace is in `biocypher-tui/`; its package retains the historical `biocypher-backend` name. Within `biocypher-tui/backend/src/`, `tui/` contains the interface, `workbench.rs` implements the operations, and `dna/`, `safety/`, and `plasmid.rs` implement the local formats and analysis.

[MIT license](LICENSE)
