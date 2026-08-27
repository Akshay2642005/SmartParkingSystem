#!/usr/bin/env bash
# Run a Wokwi headless scenario (replaces `make scenario`).
#
# wokwi-cli expects wokwi.toml in the current working directory.
#
# Usage:
#   ./scripts/scenario.sh                                          # default matrix
#   ./scripts/scenario.sh tests/wokwi-scenario-selftest.yaml      # selftest
#   ./scripts/scenario.sh --timeout 300000                        # custom timeout

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIRMWARE_DIR="$ROOT_DIR/firmware/esp32"

SCENARIO="${1:-$FIRMWARE_DIR/tests/wokwi-scenario-injection.yaml}"
TIMEOUT_MS="${TIMEOUT_MS:-180000}"

# Parse --timeout flag
while [[ $# -gt 0 ]]; do
    case "$1" in
        --timeout) TIMEOUT_MS="$2"; shift 2 ;;
        *) shift ;;
    esac
done

WOKWI_ARGS=(--scenario "$SCENARIO" --timeout "$TIMEOUT_MS")

if [[ -n "${WOKWI_GATEWAY_KEY:-}" ]]; then
    WOKWI_ARGS+=(--gateway-key "$WOKWI_GATEWAY_KEY")
fi

echo "==> Running scenario: $SCENARIO (timeout ${TIMEOUT_MS}ms)"
# wokwi-cli requires wokwi.toml in cwd
cd "$FIRMWARE_DIR"
wokwi-cli "${WOKWI_ARGS[@]}"
