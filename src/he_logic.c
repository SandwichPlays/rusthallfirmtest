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

const uint8_t KEY_MAP[NUM_KEYS] = {
    0x29, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, // Esc, 1-7
    0x25, 0x26, 0x27, 0x2D, 0x2E, 0x2A, 0x4C, 0x14, // 8-0, -, =, Bksp, Del, Q
    0x1A, 0x08, 0x15, 0x17, 0x1C, 0x18, 0x0C, 0x12, // W, E, R, T, Y, U, I, O
    0x13, 0x2F, 0x30, 0x31, 0x39, 0x04, 0x16, 0x07, // P, [, ], \, Caps, A, S, D
    0x09, 0x0A, 0x0B, 0x0D, 0x0E, 0x0F, 0x33, 0x34, // F, G, H, J, K, L, ;, '
    0x28, 0xE1, 0x1D, 0x1B, 0x06, 0x19, 0x05, 0x11, // Ent, LShift, Z, X, C, V, B, N
    0x10, 0x36, 0x37, 0x38, 0xE5, 0xE0, 0xE2, 0xE3, // M, ,, ., /, RShift, LCtrl, LAlt, LGui
    0x2C, 0xE6, 0x65, 0xE4, 0x52, 0x51, 0x50, 0x4F  // Space, RAlt, Menu, RCtrl, Up, Dn, L, R
};
