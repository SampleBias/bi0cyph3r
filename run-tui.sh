#!/usr/bin/env bash
# Run the Ratatui workbench from any working directory.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT/biocypher-tui"
exec cargo run --locked --bin bi0cyph3r -- "$@"
