#ifndef HE_LOGIC_H
#define HE_LOGIC_H

#include <stdint.h>
#include <stdbool.h>

#include "config.h"

typedef enum {
    DISCOVERY_WAITING,
    DISCOVERY_WAIT_RELEASE,
    DISCOVERY_DONE
} discovery_state_t;

typedef struct {
    uint16_t actuation_mm;    // centi-mm (150 = 1.5mm)
    uint16_t rt_down_mm;      // centi-mm
    uint16_t rt_up_mm;        // centi-mm
    uint16_t deadzone_top;    // centi-mm
    uint16_t deadzone_bottom; // centi-mm
} key_config_t;

typedef struct {
    uint16_t baseline;
    uint16_t max_travel;
    uint16_t last_val;
    uint16_t high_watermark;
    uint16_t low_watermark;
    bool active;
    discovery_state_t discovery_state;
    uint32_t discovery_timer;
} hall_key_t;

void hall_key_init(hall_key_t *key);
bool hall_key_tick(hall_key_t *key, uint16_t raw_adc, const key_config_t *config);
discovery_state_t hall_key_discovery_tick(hall_key_t *key, uint16_t raw_adc);

extern const uint8_t KEY_MAP[NUM_KEYS];

#endif
