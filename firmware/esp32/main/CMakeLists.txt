# debug_console.c compiles out entirely under #if when
# CONFIG_PARKING_DEBUG_INJECT is disabled (see Kconfig.projbuild), so it stays
# in SRCS unconditionally along with its UART driver dependency.
#
# The main/test/ case sources also stay in SRCS unconditionally: in release
# builds nothing references them, so linker garbage-collection removes every
# byte. The debug console's `selftest` serial command calls into them when
# CONFIG_PARKING_DEBUG_INJECT is enabled.

set(MAIN_SRCS
		"main.c"
		"debug_console.c"
		"sensors/ultrasonic.c"
		"parking/parking.c"
		"parking/parking_config.c"
		"mqtt/parking_payload.c"
		"net/wifi_sta.c"
		"test/cases_classification.c"
		"test/cases_debounce.c"
		"test/cases_recovery.c"
		"test/cases_statistics.c"
)

set(MAIN_REQUIRES
		esp_driver_gpio
		esp_driver_uart
		esp_timer
		esp_event
		esp_wifi
		esp_netif
		nvs_flash
		log
)

# MQTT is completely optional.
#
# When disabled:
#   - mqtt_publisher.c is not compiled
#   - mqtt component is not linked
#   - no MQTT symbols are present in the firmware
#
# When enabled:
#   - mqtt_publisher.c is compiled
#   - ESP-IDF mqtt component becomes a private dependency
if(CONFIG_PARKING_MQTT_ENABLE)
	list(APPEND MAIN_SRCS
				"mqtt/mqtt_publisher.c"
		)

	# esp-mqtt itself comes from main/idf_component.yml (managed component,
	# auto-wired into this component's requirements by the Component Manager).
endif()

idf_component_register(
		SRCS
				${MAIN_SRCS}

		INCLUDE_DIRS
				""
				"sensors"
				"parking"
				"mqtt"
				"net"
				"test"

		PRIV_REQUIRES
				${MAIN_REQUIRES}
)

# Selects the on-target branches of the portable test framework
# (no colors, esp_timer-based timing).
target_compile_definitions(
		${COMPONENT_LIB}
		PRIVATE
				PARKING_SELFTEST_ON_TARGET
)
