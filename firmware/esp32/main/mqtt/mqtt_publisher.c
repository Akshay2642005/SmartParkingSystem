#include "mqtt_publisher.h"

#include <stdlib.h>
#include <string.h>

#include "esp_event.h"
#include "esp_log.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/queue.h"
#include "freertos/task.h"
#include "mqtt_client.h"

#include "parking.h"
#include "parking_config.h"
#include "parking_payload.h"
#include "wifi_sta.h"

static const char* TAG = "mqtt_pub";

#define MQTT_TASK_NAME "mqtt_task"
#define MQTT_TASK_STACK_SIZE 4096
/* Below the parking task (5): telemetry may lag a scan, never stall one. */
#define MQTT_TASK_PRIORITY 4

#define MQTT_QUEUE_LENGTH 16

/* Burst absorption: notifications arriving within this window collapse into
 * a single snapshot (communication.md § State Updates). */
#define MQTT_COALESCE_WINDOW_MS 100
#define MQTT_COALESCE_POLL_MS 25

/* Periodic refresh doubles as the heartbeat; jittered so a garage full of
 * nodes does not synchronize (communication.md § Telemetry / Heartbeat). */
#define MQTT_REFRESH_PERIOD_MS 30000
#define MQTT_REFRESH_JITTER_MS 3000

/** Generous for 3-4 slots (~230 B rendered); truncation is logged, not UB. */
#define MQTT_PAYLOAD_BUF_SIZE 320

typedef struct {
    uint8_t slot_number;
    parking_state_t state;
} transition_msg_t;

static QueueHandle_t s_transitions;
static esp_mqtt_client_handle_t s_client;

/* Wire-visible shadow, written only by mqtt_task from queued notifications —
 * payload assembly therefore needs no lock and never touches domain state
 * cross-task (see concurrency note in parking.h). */
static parking_state_t s_shadow_state[PARKING_SLOT_COUNT];
static uint32_t s_shadow_changed_ms[PARKING_SLOT_COUNT];
static bool s_dirty;
static long s_last_change_ms;
static uint32_t s_seq;

/* Topic strings must outlive every API call: esp-mqtt keeps pointers into
 * them, so they live in static storage instead of the stack. */
static char s_state_topic[64];
static char s_status_topic[64];

static long now_ms(void) {
    return (long)(esp_timer_get_time() / 1000);
}

/* Parking-task context: hand the transition to the publisher and drop it on
 * overflow rather than ever blocking the scan (a dropped transition self-
 * heals at the next change or periodic refresh). */
static void on_parking_transition(const parking_transition_t* transition, void* ctx) {
    (void)ctx;

    const transition_msg_t msg = {
        .slot_number = transition->slot_number,
        .state = transition->state,
    };

    if (xQueueSend(s_transitions, &msg, 0) != pdTRUE) {
        ESP_LOGW(TAG, "Transition queue full; dropping slot %u", msg.slot_number);
    } else {
        ESP_LOGI(TAG, "Slot %u -> %s queued", msg.slot_number, parking_state_to_string(msg.state));
    }
}

static void apply_transition(const transition_msg_t* msg) {
    if (msg->slot_number < 1 || msg->slot_number > PARKING_SLOT_COUNT) {
        ESP_LOGW(TAG, "Ignoring out-of-range slot %u", msg->slot_number);

        return;
    }

    const size_t index = (size_t)(msg->slot_number - 1);
    s_shadow_state[index] = msg->state;
    s_shadow_changed_ms[index] = (uint32_t)now_ms();
    s_last_change_ms = now_ms();
    s_dirty = true;
}

static void publish_status(esp_mqtt_client_handle_t client, const char* status) {
    esp_mqtt_client_publish(client, s_status_topic, status, 0, 1, 1);
}

static void publish_snapshot(esp_mqtt_client_handle_t client) {
    parking_payload_slot_t slots[PARKING_SLOT_COUNT];

    for (size_t i = 0; i < PARKING_SLOT_COUNT; i++) {
        slots[i].number = slot_configs[i].id;
        slots[i].state = s_shadow_state[i];
        slots[i].changed_ms = s_shadow_changed_ms[i];
    }

    char buf[MQTT_PAYLOAD_BUF_SIZE];
    const int length = build_parking_payload(CONFIG_PARKING_MQTT_SECTION,
        slots,
        PARKING_SLOT_COUNT,
        ++s_seq,
        (uint32_t)now_ms(),
        buf,
        sizeof(buf));

    if (length < 0) {
        ESP_LOGE(TAG, "Payload encoding failed (%d slots)", (int)PARKING_SLOT_COUNT);

        return;
    }

    const int msg_id =
        esp_mqtt_client_publish(client, s_state_topic, buf, 0, /* qos */ 1, /* retain */ 1);

    if (msg_id < 0) {
        ESP_LOGW(TAG, "Publish while disconnected; retained snapshot will heal");
    } else {
        ESP_LOGI(TAG, "Published seq %u (%d B, msg %d)", s_seq, length, msg_id);
    }
}

static void mqtt_event_handler(void* handler_args,
    esp_event_base_t base,
    int32_t event_id,
    void* event_data) {
    (void)base;
    (void)event_data;

    esp_mqtt_client_handle_t client = (esp_mqtt_client_handle_t)handler_args;

    switch (event_id) {
        case MQTT_EVENT_CONNECTED:
            // Contract lifecycle step 3: announce liveness, then resync with
            // a fresh snapshot on EVERY connect (covers reconnects too).
            publish_status(client, "online");
            publish_snapshot(client);
            break;

        case MQTT_EVENT_DISCONNECTED:
            ESP_LOGW(TAG, "Broker disconnected; auto-reconnect active");
            break;

        default:
            break;
    }
}

static void mqtt_task(void* arg) {
    (void)arg;

    srand((unsigned)esp_timer_get_time());

    // The retry loop lives here: wifi_sta_connect() is idempotent after its
    // one-time init, so a failed join simply tries again.
    while (wifi_sta_connect() != ESP_OK) {
        ESP_LOGW(TAG, "WiFi join failed; retrying in 5 s");

        vTaskDelay(pdMS_TO_TICKS(5000));
    }

    snprintf(s_state_topic,
        sizeof(s_state_topic),
        "parking/%s/%s/state",
        CONFIG_PARKING_MQTT_SITE,
        CONFIG_PARKING_MQTT_SECTION);
    snprintf(s_status_topic,
        sizeof(s_status_topic),
        "parking/%s/%s/status",
        CONFIG_PARKING_MQTT_SITE,
        CONFIG_PARKING_MQTT_SECTION);

    const esp_mqtt_client_config_t config = {
        .broker.address.uri = CONFIG_PARKING_MQTT_URI,
        .credentials.username = CONFIG_PARKING_MQTT_USER,
        .credentials.authentication.password = CONFIG_PARKING_MQTT_PASSWORD,
        .session =
            {
                .keepalive = 30,
                // LWT: broker marks this node offline (retained) if we die.
                .last_will =
                    {
                        .topic = s_status_topic,
                        .msg = "offline",
                        .msg_len = 7,
                        .qos = 1,
                        .retain = 1,
                    },
            },
    };

    s_client = esp_mqtt_client_init(&config);
    if (s_client == NULL) {
        ESP_LOGE(TAG, "esp_mqtt_client_init failed; telemetry disabled");

        vTaskDelete(NULL);

        return;
    }

    ESP_ERROR_CHECK(
        esp_mqtt_client_register_event(s_client, ESP_EVENT_ANY_ID, mqtt_event_handler, s_client));
    ESP_ERROR_CHECK(esp_mqtt_client_start(s_client));

    TickType_t last_refresh = xTaskGetTickCount();
    TickType_t refresh_period =
        pdMS_TO_TICKS(MQTT_REFRESH_PERIOD_MS + rand() % (2 * MQTT_REFRESH_JITTER_MS) -
                      MQTT_REFRESH_JITTER_MS);

    while (1) {
        transition_msg_t msg;

        if (xQueueReceive(s_transitions, &msg, refresh_period - (xTaskGetTickCount() - last_refresh)) ==
            pdTRUE) {
            apply_transition(&msg);

            // Drain everything already queued, then hold the coalesce window
            // open briefly so a burst collapses into exactly one snapshot.
            while (xQueueReceive(s_transitions, &msg, 0) == pdTRUE) {
                apply_transition(&msg);
            }

            for (int waited_ms = 0; s_dirty && waited_ms < MQTT_COALESCE_WINDOW_MS;
                waited_ms += MQTT_COALESCE_POLL_MS) {
                vTaskDelay(pdMS_TO_TICKS(MQTT_COALESCE_POLL_MS));

                while (xQueueReceive(s_transitions, &msg, 0) == pdTRUE) {
                    apply_transition(&msg);
                }
            }

            publish_snapshot(s_client);

            s_dirty = false;
        }

        const TickType_t now = xTaskGetTickCount();
        if ((now - last_refresh) >= refresh_period) {
            last_refresh = now;
            refresh_period =
                pdMS_TO_TICKS(MQTT_REFRESH_PERIOD_MS + rand() % (2 * MQTT_REFRESH_JITTER_MS) -
                              MQTT_REFRESH_JITTER_MS);

            if (s_client != NULL) {
                publish_snapshot(s_client);
            }

            s_dirty = false;
        }
    }
}

esp_err_t parking_mqtt_publisher_start(void) {
    s_transitions = xQueueCreate(MQTT_QUEUE_LENGTH, sizeof(transition_msg_t));
    if (s_transitions == NULL) {
        return ESP_ERR_NO_MEM;
    }

    parking_set_event_observer(on_parking_transition, NULL);

    const BaseType_t created = xTaskCreate(mqtt_task,
        MQTT_TASK_NAME,
        MQTT_TASK_STACK_SIZE,
        NULL,
        MQTT_TASK_PRIORITY,
        NULL);

    return (created == pdPASS) ? ESP_OK : ESP_FAIL;
}
