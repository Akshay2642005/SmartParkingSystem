#!/usr/bin/env bash
# Run host-side parking domain unit tests (replaces `make test`).
#
# Builds the standalone CMake test project under firmware/esp32/tests/ using
# the system compiler (no ESP-IDF toolchain required), then runs each binary
# directly for Redis-style [ok]/[FAIL] output from parking_selftest.h.
#
# Usage:
#   ./scripts/test.sh              # build + run all units
#   ./scripts/test.sh --no-build   # run only (skip cmake build)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TESTS_DIR="$ROOT_DIR/firmware/esp32/tests"
BUILD_DIR="$TESTS_DIR/build"

build=true
if [[ "${1:-}" == "--no-build" ]]; then
    build=false
fi

if $build; then
    echo "==> Configuring host tests"
    cmake -S "$TESTS_DIR" -B "$BUILD_DIR" >/dev/null

    echo "==> Building host tests"
    cmake --build "$BUILD_DIR" >/dev/null
fi

echo "==> Running parking host tests"
failed=0
for binary in "$BUILD_DIR"/unit_*; do
    [ -x "$binary" ] || continue
    "$binary" || failed=$((failed + 1))
done

if [ $failed -gt 0 ]; then
    echo ""
    echo "!!! $failed unit(s) FAILED"
    exit 1
fi

echo ""
echo "==> All units passed"
