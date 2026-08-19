#include "parking.h"

#define PARKING_MIN_VALID_CM 2.0f
#define PARKING_MAX_VALID_CM 400.0f

void parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config) {
  if (slot == NULL) {
    return;
  }

  if (config != NULL) {
    slot->config = *config;
  }

  slot->state = PARKING_UNKNOWN;
  slot->distance_cm = 0.0f;
}

void parking_slot_update(parking_slot_t* slot, float distance_cm) {
  if (slot == NULL) {
    return;
  }

  slot->distance_cm = distance_cm;

  if (distance_cm < PARKING_MIN_VALID_CM || distance_cm > PARKING_MAX_VALID_CM) {
    slot->state = PARKING_ERROR;
    return;
  }

  if (distance_cm <= slot->config.occupied_threshold_cm) {
    slot->state = PARKING_OCCUPIED;
  } else if (slot->state == PARKING_OCCUPIED) {
    if (distance_cm > slot->config.free_threshold_cm) {
      slot->state = PARKING_FREE;
    }
  } else if (slot->state != PARKING_FREE) {
    slot->state = PARKING_FREE;
  }
}

void parking_slot_mark_error(parking_slot_t* slot) {
  if (slot != NULL) {
    slot->state = PARKING_ERROR;
  }
}

const char* parking_state_to_string(parking_state_t state) {
  switch (state) {
  case PARKING_UNKNOWN:
    return "UNKNOWN";

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
