#pragma once

#include "ultrasonic.h"

typedef enum {
  PARKING_FREE,
  PARKING_OCCUPIED,
  PARKING_ERROR,
} parking_state_t;

typedef struct {
  uint8_t id;
  ultrasonic_config_t sensor;
  float occupied_threshold_cm;
} parking_slot_config_t;

typedef struct {
  parking_slot_config_t config;
  parking_state_t state;
  float distance_cm;
} parking_slot_t;

void parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config);

void parking_slot_update(parking_slot_t* slot, float distance_cm);

const char* parking_state_to_string(parking_state_t state);
