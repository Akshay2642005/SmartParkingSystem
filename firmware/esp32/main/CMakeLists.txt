# debug_console.c compiles out entirely under #if when
# CONFIG_PARKING_DEBUG_INJECT is disabled (see Kconfig.projbuild), so it stays
# in SRCS unconditionally along with its UART driver dependency.
#
# The main/test/ case sources also stay in SRCS unconditionally: in release
# builds nothing references them, so linker garbage-collection removes every
# byte. The debug console's `selftest` serial command calls into them when
# CONFIG_PARKING_DEBUG_INJECT is enabled.
idf_component_register(
  SRCS
    "main.c"
    "debug_console.c"
    "sensors/ultrasonic.c"
    "parking/parking.c"
    "parking/parking_config.c"
    "mqtt/parking_payload.c"
    "test/cases_classification.c"
    "test/cases_debounce.c"
    "test/cases_recovery.c"
    "test/cases_statistics.c"
  INCLUDE_DIRS
    ""
    "sensors"
    "parking"
    "mqtt"
    "test"
  PRIV_REQUIRES
    esp_driver_gpio
    esp_driver_uart
    esp_timer
    log
)

# Selects the on-target branches of the portable test framework (no colors,
# esp_timer-based timing).
target_compile_definitions(${COMPONENT_LIB} PRIVATE PARKING_SELFTEST_ON_TARGET)
