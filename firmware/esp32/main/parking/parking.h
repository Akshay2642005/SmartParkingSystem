#pragma once

#include "ultrasonic.h"
typedef enum {
  PARKING_UNKNOWN,
  PARKING_OCCUPIED,
  PARKING_FREE,
  PARKING_ERROR,
} parking_state_t;

typedef struct {
  uint8_t id;
  ultrasonic_config_t sensor;

  float distance_cm;
  parking_state_t state;
} parking_slot_t;

parking_state_t parking_state_from_distance(float distance_cm);
const char* parking_state_to_string(parking_state_t state);

void parking_slot_init(parking_slot_t* slot, uint8_t id, const ultrasonic_config_t* sensor);
void parking_slot_update(parking_slot_t* slot, float distance_cm);
