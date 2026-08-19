#pragma once

typedef enum {
  PARKING_UNKNOWN,
  PARKING_OCCUPIED,
  PARKING_FREE,
  PARKING_ERROR,
} parking_state_t;

parking_state_t parking_state_from_distance(float distance_cm);
const char* parking_state_to_string(parking_state_t state);
