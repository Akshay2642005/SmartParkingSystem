#pragma once

/*
 * Host-test stub of ESP-IDF esp_log.h: logging is compiled out entirely.
 *
 * The macros still reference the tag argument so `static const char* TAG`
 * definitions in production sources do not trip -Wunused-variable under
 * -Werror. Format arguments are discarded by the preprocessor (never
 * evaluated), matching "logging off" runtime semantics.
 */

#define ESP_LOGE(tag, format, ...) ((void)(tag))
#define ESP_LOGW(tag, format, ...) ((void)(tag))
#define ESP_LOGI(tag, format, ...) ((void)(tag))
#define ESP_LOGD(tag, format, ...) ((void)(tag))
#define ESP_LOGV(tag, format, ...) ((void)(tag))
