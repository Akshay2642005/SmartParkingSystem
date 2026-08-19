#include "parking.h"

void parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config) {
  slot->config = *config;
  slot->state = PARKING_FREE;
  slot->distance_cm = 0.0f;
}

void parking_slot_update(parking_slot_t* slot, float distance_cm) {
  slot->distance_cm = distance_cm;

  if (distance_cm <= slot->config.occupied_threshold_cm) {
    slot->state = PARKING_OCCUPIED;
  } else {
    slot->state = PARKING_FREE;
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
