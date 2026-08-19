#include "parking.h"

#define OCCUPIED_DISTANCE_CM 30.0f

parking_state_t parking_state_from_distance(float distance_cm) {
  if (distance_cm <= 0) {
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
