#pragma once

/**
 * Debug serial injection console (see Kconfig.projbuild).
 *
 * Compiled in only when CONFIG_PARKING_DEBUG_INJECT is enabled; otherwise
 * debug_console_start() is a no-op so callers need no preprocessor guards.
 */
#if CONFIG_PARKING_DEBUG_INJECT

/** Start the UART0 reader task handling "setdist" commands. Fatal on failure. */
void debug_console_start(void);

#else

static inline void debug_console_start(void) {}

#endif /* CONFIG_PARKING_DEBUG_INJECT */
