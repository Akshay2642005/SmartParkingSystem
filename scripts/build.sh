#!/usr/bin/env bash
# Build the ESP32 firmware (replaces `make build`).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIRMWARE_DIR="$ROOT_DIR/firmware/esp32"

ESP_IDF="${ESP_IDF:-$HOME/.espressif/v6.0.2/esp-idf}"

if [[ ! -f "$ESP_IDF/export.sh" ]]; then
    echo "Error: IDF not found at $ESP_IDF"
    echo "Set ESP_IDF env var or install ESP-IDF v6.0.2"
    exit 1
fi

# shellcheck disable=SC1091
. "$ESP_IDF/export.sh" >/dev/null

if [[ "${1:-}" == "--clean" ]]; then
    echo "==> Cleaning build directory"
    rm -rf "$FIRMWARE_DIR/build"
fi

if [[ ! -f "$FIRMWARE_DIR/build/CMakeCache.txt" ]]; then
    echo "==> Configuring build tree"
    cd "$FIRMWARE_DIR"
    idf.py reconfigure
fi

echo "==> Building firmware"
cd "$FIRMWARE_DIR"
idf.py build
