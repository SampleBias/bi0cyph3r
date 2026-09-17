#!/usr/bin/env bash
# Bi0cyph3r — build and optionally install the terminal workbench
# Usage: ./install.sh [ -i ]   ( -i = install to ~/.local/bin )
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT/biocypher-rust-solana"

echo "🧬 Building Bi0cyph3r..."
cargo build --locked --release --bin bi0cyph3r

BIN="$REPO_ROOT/biocypher-rust-solana/target/release/bi0cyph3r"
echo ""
echo "✅ Build complete!"
echo ""
echo "  Terminal app: $BIN"

if [[ "${1:-}" == "-i" ]]; then
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
  cp "$BIN" "$INSTALL_DIR/bi0cyph3r"
  chmod +x "$INSTALL_DIR/bi0cyph3r"
  echo ""
  echo "  Installed to $INSTALL_DIR/bi0cyph3r"
  echo "  (ensure $INSTALL_DIR is in your PATH)"
fi

echo ""
echo "Usage:"
echo "  bi0cyph3r                     # Open the Ratatui workbench"
echo "  bi0cyph3r encode \"Hello World\""
echo "  bi0cyph3r decode \"TACATCTTTCG...\""
echo "  bi0cyph3r safety \"ATCGATCG\""
echo ""
echo "  ./run-tui.sh                  # Run from this checkout"
