# debug_console.c compiles out entirely under #if when
# CONFIG_PARKING_DEBUG_INJECT is disabled (see Kconfig.projbuild), so it stays
# in SRCS unconditionally along with its UART driver dependency.
idf_component_register(
  SRCS
    "main.c"
    "debug_console.c"
    "sensors/ultrasonic.c"
    "parking/parking.c"
    "parking/parking_config.c"
  INCLUDE_DIRS
    ""
    "sensors"
    "parking"
  PRIV_REQUIRES
    esp_driver_gpio
    esp_driver_uart
    esp_timer
    log
)
