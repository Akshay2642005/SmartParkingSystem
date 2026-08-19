# ADR-0002: ESP-IDF

Status: Accepted

## Context

The device layer is built on the ESP32. The firmware repository
(`firmware/esp32/`) uses a CMake-based ESP-IDF project (`project(smart_parking)`
in `CMakeLists.txt`) and targets ESP32-family chips.

## Problem

Which firmware framework and toolchain should the ESP32 firmware use?

## Decision

Use **ESP-IDF** (Espressif's official framework) for the firmware.

This is confirmed by the repository:

- `firmware/esp32/CMakeLists.txt` includes `${IDF_PATH}/tools/cmake/project.cmake`
  and declares `project(smart_parking)`.
- `firmware/esp32/main/CMakeLists.txt` uses `idf_component_register`.
- `firmware/esp32/main/hello_world_main.c` uses FreeRTOS (`freertos/task.h`) and
  native ESP32 APIs (`esp_chip_info`, `esp_flash_get_size`).
- `firmware/esp32/.devcontainer/` is based on the `espressif/idf` image.

## Decision Details

- **Framework**: ESP-IDF
- **RTOS**: FreeRTOS (provided by ESP-IDF)
- **Language**: C (current source uses C)
- **Build system**: CMake-based ESP-IDF build
- **Tooling**: `idf.py`, devcontainer with ESP-IDF image

## Alternatives Considered

- **Arduino framework**: rejected — the repository already establishes ESP-IDF;
  Arduino would introduce a parallel abstraction layer without benefit.

## Consequences

### Positive

- Native ESP32 APIs and FreeRTOS integration.
- Full control over low-level hardware and Wi-Fi.
- Mature build system and tooling.
- Established by the existing repository (no speculative technology).

### Negative

- More verbose than Arduino for simple sketches.
- Requires ESP-IDF toolchain and environment.

## Validation

- Build configuration present in `CMakeLists.txt` files.
- Development container uses the `espressif/idf` image.
- Source compiles against ESP-IDF headers.

## Related Documents

- `../architecture/firmware-architecture.md`
- `../architecture/tech-stack.md`
- `../decisions/ADR-0003-wokwi.md`