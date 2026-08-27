#!/usr/bin/env bash
# Build one firmware binary per parking section and stage them (plus a shared
# diagram.json) under out/ so each can be dropped into its own Wokwi project.
#
# The firmware bakes the section id in from Kconfig (CONFIG_PARKING_MQTT_SECTION),
# so a section is really a separate build with a different sdkconfig. We patch
# sdkconfig per section (kconfiglib, sed fallback) and rebuild each time.
#
# Usage:
#   ./scripts/build-sections.sh                # sections A B C, site "main"
#   ./scripts/build-sections.sh A B C D        # custom section list
#   SITE=garage ./scripts/build-sections.sh B C
#   OUT_DIR=out/sections ./scripts/build-sections.sh A B
#
# Output (under OUT_DIR):
#   diagram.json          shared (all sections use the same 3-sensor wiring)
#   sectionA.elf  sectionA.bin
#   sectionB.elf  sectionB.bin
#   ...
#
# Then in Wokwi: create one project per section, paste the same diagram.json,
# and upload that section's .elf (or .bin) as the firmware. The server shows
# every section automatically under parking/{site}/{section}/state.

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

SITE="${SITE:-main}"
OUT_DIR="${OUT_DIR:-$ROOT_DIR/out}"

SECTIONS=("$@")
if [[ ${#SECTIONS[@]} -eq 0 ]]; then
    SECTIONS=(A B C)
fi

mkdir -p "$OUT_DIR"

# Ensure an sdkconfig exists so the section/symbol is present before we edit it.
if [[ ! -f "$FIRMWARE_DIR/sdkconfig" ]]; then
    echo "==> Generating initial sdkconfig"
    ( cd "$FIRMWARE_DIR" && idf.py reconfigure >/dev/null )
fi

# Write CONFIG_PARKING_MQTT_SECTION / _SITE into sdkconfig non-interactively.
# Prefer kconfiglib (bundled with ESP-IDF); fall back to sed (edits only).
set_section() {
    local section="$1" site="$2"
    if python - "$FIRMWARE_DIR" "$section" "$site" <<'PY'
import sys, os
base, section, site = sys.argv[1], sys.argv[2], sys.argv[3]
cfg = os.path.join(base, "sdkconfig")
lines = open(cfg).read().splitlines() if os.path.exists(cfg) else []

def set_or_add(key, val):
    target = key + "="
    for i, ln in enumerate(lines):
        if ln.startswith(target):
            lines[i] = '%s="%s"' % (key, val)
            return
    lines.append('%s="%s"' % (key, val))

set_or_add("CONFIG_PARKING_MQTT_SECTION", section)
set_or_add("CONFIG_PARKING_MQTT_SITE", site)
open(cfg, "w").write("\n".join(lines) + "\n")
PY
    then
        return 0
    fi
    sed -i.bak -E \
        -e "s|^CONFIG_PARKING_MQTT_SECTION=.*|CONFIG_PARKING_MQTT_SECTION=\"$section\"|" \
        -e "s|^CONFIG_PARKING_MQTT_SITE=.*|CONFIG_PARKING_MQTT_SITE=\"$site\"|" \
        "$FIRMWARE_DIR/sdkconfig"
}

for SECTION in "${SECTIONS[@]}"; do
    echo "==> Building section $SECTION (site=$SITE)"
    set_section "$SECTION" "$SITE"
    ( cd "$FIRMWARE_DIR" && idf.py reconfigure >/dev/null && idf.py build >/dev/null )

    cp "$FIRMWARE_DIR/build/smart_parking.elf" "$OUT_DIR/section${SECTION}.elf"
    cp "$FIRMWARE_DIR/build/smart_parking.bin" "$OUT_DIR/section${SECTION}.bin"
    echo "    staged: out/section${SECTION}.elf / .bin"
done

cp "$FIRMWARE_DIR/diagram.json" "$OUT_DIR/diagram.json"
echo "==> Done. Staged diagram.json + ${#SECTIONS[@]} section builds in $OUT_DIR"
