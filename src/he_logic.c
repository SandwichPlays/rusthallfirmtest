#include "he_logic.h"

void hall_key_init(hall_key_t *key) {
    key->baseline = 0;
    key->max_travel = 0;
    key->last_val = 0;
    key->high_watermark = 0;
    key->low_watermark = 0;
    key->active = false;
    key->discovery_state = DISCOVERY_WAITING;
    key->discovery_timer = 0;
}

discovery_state_t hall_key_discovery_tick(hall_key_t *key, uint16_t raw_adc) {
    switch (key->discovery_state) {
        case DISCOVERY_WAITING:
            if (key->baseline == 0) key->baseline = raw_adc;
            // Baseline tracking (slow average)
            key->baseline = (key->baseline * 31 + raw_adc) / 32;
            
            if (raw_adc > key->baseline + 200) {
                key->discovery_state = DISCOVERY_WAIT_RELEASE;
                key->max_travel = raw_adc;
            }
            break;
            
        case DISCOVERY_WAIT_RELEASE:
            if (raw_adc > key->max_travel) key->max_travel = raw_adc;
            if (raw_adc < key->baseline + 100) {
                key->discovery_timer++;
                if (key->discovery_timer > 1000) { // 1 second at 1kHz
                    key->discovery_state = DISCOVERY_DONE;
                }
            } else {
                key->discovery_timer = 0;
            }
            break;
            
        case DISCOVERY_DONE:
            break;
    }
    return key->discovery_state;
}

bool hall_key_tick(hall_key_t *key, uint16_t raw_adc, const key_config_t *config) {
    if (key->max_travel <= key->baseline) return false;
    
    // Normalize to 0-400 (centi-mm travel, assuming 4.0mm range)
    int32_t range = key->max_travel - key->baseline;
    int32_t current_mm = ((int32_t)raw_adc - key->baseline) * 400 / range;
    if (current_mm < 0) current_mm = 0;
    if (current_mm > 400) current_mm = 400;

    if (!key->active) {
        if (current_mm > config->actuation_mm + config->deadzone_top) {
            key->active = true;
            key->low_watermark = current_mm;
        }
    } else {
        if (current_mm < config->deadzone_top) {
            key->active = false;
            key->high_watermark = current_mm;
        } else {
            // Rapid Trigger Logic
            if (current_mm > key->high_watermark) {
                key->high_watermark = current_mm;
                if (current_mm > key->low_watermark + config->rt_down_mm) {
                    key->active = true;
                    key->low_watermark = current_mm;
                }
            }
            
            if (current_mm < key->low_watermark) {
                key->low_watermark = current_mm;
                if (current_mm < key->high_watermark - config->rt_up_mm) {
                    key->active = false;
                    key->high_watermark = current_mm;
                }
            }
        }
    }

    key->last_val = current_mm;
    return key->active;
}

const uint8_t KEY_MAP[NUM_KEYS] = MY_KEY_MAP;
