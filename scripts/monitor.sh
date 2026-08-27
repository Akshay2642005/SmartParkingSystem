#!/usr/bin/env bash
# Attach serial console (replaces `make monitor`).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIRMWARE_DIR="$ROOT_DIR/firmware/esp32"

ESP_IDF="${ESP_IDF:-$HOME/.espressif/v6.0.2/esp-idf}"
PORT="${PORT:-/dev/cu.usbserial-0001}"

# shellcheck disable=SC1091
. "$ESP_IDF/export.sh" >/dev/null

cd "$FIRMWARE_DIR"
idf.py -p "$PORT" monitor
