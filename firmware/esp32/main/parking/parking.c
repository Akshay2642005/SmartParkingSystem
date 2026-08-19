#include "parking.h"

#define OCCUPIED_DISTANCE_CM 30.0f

parking_state_t parking_state_from_distance(float distance_cm) {
  if (distance_cm <= 0.0f) {
    return PARKING_ERROR;
  }
  if (distance_cm <= OCCUPIED_DISTANCE_CM) {
    return PARKING_OCCUPIED;
  }
  return PARKING_FREE;
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

void parking_slot_init(parking_slot_t* slot, uint8_t id, const ultrasonic_config_t* sensor) {
  if (slot == NULL || sensor == NULL) {
    return;
  }
  slot->id = id;
  slot->sensor = *sensor;
  slot->distance_cm = 0.0f;
  slot->state = PARKING_UNKNOWN;
}

void parking_slot_update(parking_slot_t* slot, float distance_cm) {
  if (slot == NULL) {
    return;
  }
  slot->distance_cm = distance_cm;
  slot->state = parking_state_from_distance(distance_cm);
}
