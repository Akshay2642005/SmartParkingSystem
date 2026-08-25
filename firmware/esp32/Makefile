# Developer entry points for the ESP32 parking-sensor firmware.
#
# Everything wraps CMake/Ninja (ESP-IDF v6 toolchain) — nothing is built by
# make itself. The firmware targets share the exact same build/ directory as
# `idf.py`, so both entry points stay interchangeable.
#
#   make             incremental build, like `cmake --build build`
#                    (a CMakeLists.txt change triggers automatic
#                     reconfiguration before compiling)
#   make configure   one-shot full reconfigure (= idf.py reconfigure)
#   make flash       flash over serial                   (= idf.py flash)
#   make monitor     attach serial console               (= idf.py monitor)
#   make up          flash + monitor
#   make menuconfig  Kconfig editor                      (= idf.py menuconfig)
#   make size        firmware size breakdown             (= idf.py size)
#   make test        FULL gate: host tests
#   make scenario    one headless Wokwi run (SCENARIO=<file>, default matrix)
#   make clean       wipe both firmware and host-test build directories
#   make env 				 source esp-id env
#
# Override on the command line when needed:
#   make ESP_IDF=~/.espressif/v9.9.9/esp-idf flash PORT=/dev/cu.usbmodemX

ESP_IDF  ?= $(HOME)/.espressif/v6.0.2/esp-idf
PORT     ?= /dev/cu.usbserial-0001
BAUD     ?= 115200
SCENARIO ?= tests/wokwi-scenario-injection.yaml

# wokwi-cli --timeout is in MILLISECONDS.
WOKWI_TIMEOUT_MS ?= 180000

# Every scenario must pass for `make test` to succeed.
SCENARIOS := \
    tests/wokwi-scenario.yaml \
    tests/wokwi-scenario-injection.yaml \
    tests/wokwi-scenario-selftest.yaml

HOST_DIR := tests
HOST_BUILD := tests/build
BUILD := build

idf := . $(ESP_IDF)/export.sh >/dev/null && idf.py

.DEFAULT_GOAL := build
.PHONY: configure build flash monitor up menuconfig size test test-host scenario clean env

# One-shot full reconfigure; also the bootstrap for a fresh clone.
configure:
	@$(idf) reconfigure

# Fast path: let ninja decide what to rebuild. Ninja reruns CMake by itself
# whenever a listed CMakeLists.txt/Kconfig input changed, so no explicit
# reconfigure is needed here. A missing build tree — or one polluted by a
# foreign generator (e.g. an IDE configuring with Unix Makefiles) — falls
# back to a full idf.py reconfigure. IDF is activated quietly on every run:
# nested configure steps (the bootloader subproject) need its tools on PATH,
# and each recipe line runs in its own shell, so sourcing belongs on the
# build line itself.
build:
	@if [ ! -f $(BUILD)/CMakeCache.txt ] || \
	    ! grep -q '^CMAKE_GENERATOR:INTERNAL=Ninja$$' $(BUILD)/CMakeCache.txt; then \
	    echo '==> (Re)configuring build tree'; \
	    $(idf) reconfigure; \
	fi
	@. $(ESP_IDF)/export.sh >/dev/null && cmake --build $(BUILD)

flash:
	@$(idf) -p $(PORT) -b $(BAUD) flash

monitor:
	@$(idf) -p $(PORT) -b $(BAUD) monitor

up:
	@$(idf) -p $(PORT) -b $(BAUD) flash monitor

menuconfig:
	@$(idf) menuconfig

size:
	@$(idf) size

# Fast inner loop: host-side parking-domain units only — production sources
# compiled against stub drivers, Redis-style output, sub-second, no hardware.
test:
	@cmake -S $(HOST_DIR) -B $(HOST_BUILD) >/dev/null
	@cmake --build $(HOST_BUILD) >/dev/null
	@echo "==> Running parking host tests"
	@failed=0; \
	for binary in $(HOST_BUILD)/unit_*; do \
	    [ -x "$$binary" ] || continue; \
	    echo "Testing unit/$${binary##*/unit_}"; \
	    "$$binary" || failed=$$((failed + 1)); \
	done; \
	if [ $$failed -gt 0 ]; then \
	    echo "!!! $$failed unit(s) FAILED"; \
	    exit 1; \
	fi

# Optional Wokwi Private IoT Gateway key: wokwi-cli sims have no network
# access without one. Set via env when a broker test needs real WiFi:
#   make scenario WOKWI_GATEWAY_KEY=$(WOKWI_IOT_GATEWAY_KEY)
WOKWI_GATEWAY_KEY ?=

scenario:
	@if [ -n "$(WOKWI_GATEWAY_KEY)" ]; then \
		wokwi-cli --scenario $(SCENARIO) --timeout $(WOKWI_TIMEOUT_MS) \
			--gateway-key "$(WOKWI_GATEWAY_KEY)"; \
	else \
		wokwi-cli --scenario $(SCENARIO) --timeout $(WOKWI_TIMEOUT_MS); \
	fi

clean:
	@rm -rf $(HOST_BUILD)
	@rm -rf $(BUILD)

env:
	@source ~/.espressif/tools/activate_idf_v6.0.2.sh

