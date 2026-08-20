#pragma once

#include <stddef.h>
#include <stdint.h>

#include "esp_err.h"
#include "ultrasonic.h"

typedef enum {
  PARKING_FREE,
  PARKING_OCCUPIED,
  PARKING_ERROR,
} parking_state_t;

typedef enum {
  PARKING_EVENT_NONE,
  PARKING_EVENT_SLOT_OCCUPIED,
  PARKING_EVENT_SLOT_FREED,
} parking_event_t;

typedef struct {
  uint8_t id;
  ultrasonic_config_t sensor;
  float occupied_threshold_cm;
  float free_threshold_cm;
} parking_slot_config_t;

typedef struct {
  parking_slot_config_t config;
  ultrasonic_sensor_t sensor;
  parking_state_t state;
  float distance_cm;
} parking_slot_t;

typedef struct {
  parking_slot_t* slots;
  size_t slot_count;
  size_t occupied_count;
  size_t available_count;
} parking_lot_t;

esp_err_t parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config);

parking_event_t parking_slot_update(parking_slot_t* slot, float distance_cm);

void parking_lot_init(parking_lot_t* lot, parking_slot_t* slots, size_t slot_count);
void parking_lot_update_counts(parking_lot_t* lot);

size_t parking_lot_get_total(const parking_lot_t* lot);
size_t parking_lot_get_occupied(const parking_lot_t* lot);
size_t parking_lot_get_available(const parking_lot_t* lot);

const char* parking_state_to_string(parking_state_t state);
