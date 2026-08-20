#include "parking.h"

#include "esp_log.h"

static const char* TAG = "parking";

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

void parking_slot_mark_error(parking_slot_t* slot) {
  if (slot != NULL) {
    slot->state = PARKING_ERROR;
  }
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

void parking_lot_init(parking_lot_t* lot, parking_slot_t* slots, size_t slot_count) {
  if (lot == NULL) {
    return;
  }

  lot->slots = slots;
  lot->slot_count = slot_count;
  lot->occupied_count = 0;
  lot->available_count = slot_count;

  parking_lot_update_counts(lot);
}

void parking_lot_update_counts(parking_lot_t* lot) {
  if (lot == NULL || lot->slots == NULL) {
    return;
  }

  lot->occupied_count = 0;

  for (size_t i = 0; i < lot->slot_count; i++) {
    if (lot->slots[i].state == PARKING_OCCUPIED) {
      lot->occupied_count++;
    }
  }

  lot->available_count = lot->slot_count - lot->occupied_count;
}

esp_err_t parking_lot_scan(parking_lot_t* lot) {
  if (lot == NULL || lot->slots == NULL) {
    return ESP_ERR_INVALID_ARG;
  }

  for (size_t i = 0; i < lot->slot_count; i++) {
    parking_slot_t* slot = &lot->slots[i];
    float distance_cm;

    esp_err_t result = ultrasonic_measure_cm(&slot->sensor, &distance_cm);

    if (result != ESP_OK) {
      parking_slot_mark_error(slot);
      ESP_LOGE(TAG, "Slot %d | Sensor error: %s", slot->config.id, esp_err_to_name(result));
      continue;
    }

    parking_event_t event = parking_slot_update(slot, distance_cm);

    switch (event) {
      case PARKING_EVENT_SLOT_OCCUPIED:
        ESP_LOGI(TAG, "Slot %d became OCCUPIED", slot->config.id);
        break;

      case PARKING_EVENT_SLOT_FREED:
        ESP_LOGI(TAG, "Slot %d became FREE", slot->config.id);
        break;

      case PARKING_EVENT_NONE:
        break;
    }

    ESP_LOGD(TAG,
      "Slot %d | Distance: %.2f cm | State: %s",
      slot->config.id,
      slot->distance_cm,
      parking_state_to_string(slot->state));
  }

  parking_lot_update_counts(lot);

  ESP_LOGI(TAG,
    "Parking Lot | Total: %zu | Occupied: %zu | Available: %zu",
    parking_lot_get_total(lot),
    parking_lot_get_occupied(lot),
    parking_lot_get_available(lot));

  return ESP_OK;
}

size_t parking_lot_get_total(const parking_lot_t* lot) {
  if (lot == NULL) {
    return 0;
  }

  return lot->slot_count;
}

size_t parking_lot_get_occupied(const parking_lot_t* lot) {
  if (lot == NULL) {
    return 0;
  }

  return lot->occupied_count;
}

size_t parking_lot_get_available(const parking_lot_t* lot) {
  if (lot == NULL) {
    return 0;
  }

  return lot->available_count;
}