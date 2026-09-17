# Bi0cyph3r

A terminal DNA workbench built with [Ratatui](https://ratatui.rs/). Encode messages, decode sequences, inspect DNA, and prepare plasmid payloads in a local, keyboard-driven interface.

The app is terminal-only. It starts no web server and requires no browser, GUI, interpreter, or network connection for local work.

## Launch

Install a current stable Rust toolchain (Rust 1.88 or later), then run:

```bash
./run-tui.sh
```

Or build and install the standalone binary:

```bash
./install.sh          # Build the release binary
./install.sh -i       # Also install into ~/.local/bin
bi0cyph3r             # Open the TUI
bi0cyph3r tui         # Explicit equivalent
```

From the Rust workspace, `cargo run` also opens the TUI. A terminal of at least 64 × 22 is required; 120 × 36 or larger gives the full sidebar layout.

## Start screen & themes

The TUI opens with the Bi0cyph3r ASCII logo and an animated double helix rotating around its vertical axis. The start menu offers **s — start**, **g — guide**, and **t — themes**. You can also select with the arrow keys / Tab and open with Enter. Space pauses or resumes the animation.

The themes screen previews five palettes across the whole interface:

| Theme | Palette |
| --- | --- |
| Original | The original dark cyan and violet |
| Crimson | Red and scarlet on a dark background |
| Paper | White background with dark text |
| Monochrome | Black, white, and gray |
| Amber | Warm vintage phosphor |

Use arrows or `1`–`5` to preview, Enter to apply, or Esc to cancel. Theme choices last for the current session. Set a preferred launch palette with `bi0cyph3r --theme paper` or `BIOCYPHER_THEME=paper bi0cyph3r`; valid names are `original`, `crimson`, `paper`, `monochrome`, and `amber`.

In the workbench, `t` / `F9` opens themes and `g` opens the guide. Esc leaves editing; Esc again returns to the start screen while preserving your session. The landing screen carries the credit: **Made with ASCII Heart by S4MPL3BI4S <3**.

## Workbench

Four workspaces retain their inputs and last successful results for the current session:

| Workspace | What it does |
| --- | --- |
| Encode | Convert UTF-8 text to DNA using Basic, Nanopore, Secure, or Split Key mode |
| Decode | Recover text from raw DNA or a single FASTA record; extract marked plasmid payloads |
| Safety | Inspect GC content, signature matches, natural matches, homopolymers, and reading frames |
| Plasmid | Name and assemble a payload, optionally with decoding markers or the existing eGFP cassette; export sequence and annotations |

The interface includes color-coded bases, sequence coordinates, GC meters, masked credential fields, session activity, background processing, and a built-in keyboard guide.

| Key | Action |
| --- | --- |
| `1`–`4` / `F1`–`F4` | Switch workspace; function keys also work while editing |
| `Tab` / `Shift+Tab` | Focus the next / previous field |
| `i` / `Enter` | Edit the focused field |
| `Esc` | Finish editing, close a dialog, or return to the start screen |
| `t` / `F9` | Preview and switch themes |
| `g` | Open the scrollable guide |
| `m` | Cycle encoding mode |
| `x` | Cycle plasmid structure |
| `Ctrl+R` / `F5` | Run |
| `Ctrl+O` | Import a UTF-8 text or single-record FASTA file |
| `Ctrl+S` / `F6` | Export; `Tab` cycles TXT, FASTA, JSON in the dialog |
| `Ctrl+K` / `v` | Export / reveal generated split keys |
| `d` / `s` | Send the result to Decode / Safety |
| `PgUp` / `PgDn` | Scroll the result |
| `Ctrl+U` | Clear the field being edited |
| `?` | Show keyboard help |
| `q` / `Ctrl+Q` / `F10` | Quit; `Ctrl+C` also works while editing |

Letter shortcuts apply in normal mode. While editing, letters are input, Enter inserts a newline in the message field, and arrows/Home/End move the cursor. Bracketed paste is supported.

Press `s` on the start screen, then `i`, type a message, and press `Ctrl+R`. Press `d` and `Ctrl+R` to decode the result. Running an operation leaves editing mode automatically.

## Scriptable commands

The same operations remain available for shell pipelines:

```bash
bi0cyph3r encode "Hello World"
bi0cyph3r encode "Hello World" | bi0cyph3r decode -
bi0cyph3r encode --input message.txt --save sequence.dna
bi0cyph3r decode --input sequence.dna
bi0cyph3r safety ATCGATCGATCG --output json
bi0cyph3r plasmid "Hello" --name sample --save sample.fasta
bi0cyph3r plasmid "Hello" --structure marked --output json
bi0cyph3r --help
```

Use `--mode basic|nanopore|secure|splitkey`. Supply secure-mode passwords through the masked TUI field or `BIOCYPHER_PASSWORD`; `--password` is also supported. The CLI accepts piped input, `--input FILE`, or a positional argument.

Split-key encoding requires an explicit key export:

```bash
bi0cyph3r encode "Secret" --mode splitkey --keys-output keys.json --save secret.dna
bi0cyph3r decode --input secret.dna --mode splitkey --k1 BASE64_K1 --k2 BASE64_K2
```

Decoding also accepts `BIOCYPHER_K1` and `BIOCYPHER_K2`. Ordinary TXT, FASTA, and JSON result exports omit split keys. The explicit key bundle contains both keys; store them separately afterward. The TUI asks before quitting or replacing a result containing unexported keys.

Files are created without overwriting existing paths, with owner-only permissions on Unix. Session data is held in memory and disappears on exit unless exported. Messages are limited to 4 KiB and sequence imports to 256 KiB, keeping generated DNA within the import limit. DNA input accepts whitespace and lowercase bases, but rejects invalid symbols and multiple FASTA records.

## Modes and limits

- Basic maps UTF-8 bytes to A/T/C/G. It is encoding, not encryption.
- Nanopore uses the existing triplet, parity, redundancy, and padding format.
- Secure retains the existing AES-256-CBC/PBKDF2 format.
- Split Key retains the existing encryption and two-share key format.

The existing CBC formats do not provide authenticated encryption. The safety report is a local heuristic, not a biological safety certification. Plasmid assembly retains the former designer's sequence parts; it does not validate a complete expression-ready vector.

## Docker

```bash
docker compose build biocypher
docker compose run --rm --user "$(id -u):$(id -g)" biocypher
docker compose run --rm --user "$(id -u):$(id -g)" biocypher encode "Hello"
```

Compose mounts this checkout at `/data` for importing and exporting files. Running with your user ID keeps exported files owned by you. There are no exposed ports.

## Development

```bash
cd biocypher-rust-solana
cargo fmt --all --check
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The Rust package retains its historical `biocypher-backend` library name. `backend/src/tui/` contains the application, editor, and rendering; `workbench.rs` is shared by the TUI and CLI; `dna/`, `safety/`, and `plasmid.rs` provide local operations.

All application code and tests are Rust. The [historical protocol notes](docs/PROTOCOL_SPECIFICATION.md) and [original attribution and license](docs/LEGACY_LICENSE) are preserved. CLI integration tests run with an empty `PATH` to check that local operations do not rely on an interpreter or other external tools.

Export files for external delivery; no automatic escrow or manufacturer transmission is performed. There is no web server, GUI, browser wallet, or blockchain integration.

[MIT license](LICENSE)
