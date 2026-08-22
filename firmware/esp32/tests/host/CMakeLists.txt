cmake_minimum_required(VERSION 3.16)
project(parking_host_tests LANGUAGES C)

# Host-side unit tests for the parking domain (Phase 20).
#
# The real production sources are compiled against tiny stubs of the ESP-IDF
# surface (esp_err.h, esp_log.h, driver/gpio.h) and a scriptable stand-in for
# the ultrasonic driver. No IDF toolchain, no hardware: plain C11 on the
# host, driven by CTest:
#
#   make test                          # from firmware/esp32 — Redis-style output
#   ctest --test-dir tests/host/build  # same binaries via CTest (CI)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(MAIN_DIR ${CMAKE_CURRENT_LIST_DIR}/../../main)

add_library(parking_domain STATIC
    ${MAIN_DIR}/parking/parking.c
    ${MAIN_DIR}/parking/parking_config.c
    stubs/ultrasonic_stub.c
)
target_include_directories(parking_domain PUBLIC
    ${MAIN_DIR}/parking
    ${MAIN_DIR}/sensors
    ${CMAKE_CURRENT_LIST_DIR}/stubs
)
# CONFIG_PARKING_DEBUG_INJECT is intentionally undefined here: the debug
# injection hook compiles out exactly like in a release firmware build.
target_compile_options(parking_domain PRIVATE -Wall -Wextra -Werror)

enable_testing()

set(PARKING_TEST_UNITS classification debounce errors recovery statistics)

foreach(unit IN LISTS PARKING_TEST_UNITS)
  add_executable(unit_${unit} units/unit_${unit}.c)
  target_link_libraries(unit_${unit} PRIVATE parking_domain)
  target_include_directories(unit_${unit} PRIVATE ${CMAKE_CURRENT_LIST_DIR})
  target_compile_options(unit_${unit} PRIVATE -Wall -Wextra -Werror)
  add_test(NAME unit/${unit} COMMAND unit_${unit})
endforeach()

set_tests_properties(
    unit/classification
    unit/debounce
    unit/errors
    unit/recovery
    unit/statistics
    PROPERTIES TIMEOUT 60
)
