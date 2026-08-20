#include "parking.h"

esp_err_t parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config) {

  if (slot == NULL || config == NULL) {
    return ESP_ERR_INVALID_ARG;
  }
  slot->config = *config;
  slot->state = PARKING_FREE;
  slot->distance_cm = 0.0f;

  return ultrasonic_init(&slot->sensor, &slot->config.sensor);
}

parking_event_t parking_slot_update(parking_slot_t* slot, float distance_cm) {
  if (slot == NULL) {
    return PARKING_EVENT_NONE;
  }
  slot->distance_cm = distance_cm;

  switch (slot->state) {
    case PARKING_FREE:
      if (distance_cm <= slot->config.occupied_threshold_cm) {
        slot->state = PARKING_OCCUPIED;
        return PARKING_EVENT_SLOT_OCCUPIED;
      }
      break;

    case PARKING_OCCUPIED:
      if (distance_cm >= slot->config.free_threshold_cm) {
        slot->state = PARKING_FREE;
        return PARKING_EVENT_SLOT_FREED;
      }
      break;

    case PARKING_ERROR:
      break;
  }
  return PARKING_EVENT_NONE;
}

const char* parking_state_to_string(parking_state_t state) {
  switch (state) {
    case PARKING_FREE:
      return "FREE";

    case PARKING_OCCUPIED:
      return "OCCUPIED";

    case PARKING_ERROR:
      return "ERROR";

    default:
      return "UNKNOWN";
  }
}
