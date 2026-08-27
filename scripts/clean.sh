#!/usr/bin/env bash
# Wipe all build directories (replaces `make clean`).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> Removing firmware build directory"
rm -rf "$ROOT_DIR/firmware/esp32/build"

echo "==> Removing host test build directory"
rm -rf "$ROOT_DIR/firmware/esp32/tests/build"

echo "==> Done"
