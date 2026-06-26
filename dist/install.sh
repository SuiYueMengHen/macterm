#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="macterm"
DEFAULT_INSTALL_DIR="/usr/local/bin"
BIN_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

# Resolve script dir (support symlinks)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$SCRIPT_DIR/$BIN_NAME"

if [ ! -f "$SRC" ]; then
  echo "Error: $BIN_NAME not found next to install.sh" >&2
  exit 1
fi

if [ ! -d "$BIN_DIR" ]; then
  echo "Creating $BIN_DIR ..."
  mkdir -p "$BIN_DIR"
fi

echo "Installing $BIN_NAME to $BIN_DIR ..."
install -m 755 "$SRC" "$BIN_DIR/$BIN_NAME"

echo "✓ Installed $BIN_NAME to $BIN_DIR/$BIN_NAME"
echo ""
echo "Run with: macterm"
echo "  macterm        # start with 1 pane"
echo "  macterm -f     # start with file tree"
echo "  macterm -n 4   # start with 4 panes"
echo "  macterm --help # show all options"
