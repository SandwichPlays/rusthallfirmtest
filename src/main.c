#include "at32f405.h"
#include "he_logic.h"
#include "tusb.h"

// Hardware hooks from hw.c
void init_clocks(void);
void init_adc_dma(uint16_t *buffer, uint16_t len);
void init_rgb(uint16_t *buffer, uint16_t len);
void update_rgb(uint16_t len);

static uint16_t adc_buffer[NUM_KEYS];
static uint16_t rgb_buffer[NUM_KEYS * 24 + 1];
static hall_key_t keys[NUM_KEYS];

void set_led(int index, uint8_t r, uint8_t g, uint8_t b) {
    uint16_t *base = &rgb_buffer[index * 24];
    for (int i = 0; i < 8; i++) {
        base[i] = (g & (1 << (7 - i))) ? 18 : 9;
        base[8 + i] = (r & (1 << (7 - i))) ? 18 : 9;
        base[16 + i] = (b & (1 << (7 - i))) ? 18 : 9;
    }
}

int main(void) {
    init_clocks();
    init_adc_dma(adc_buffer, NUM_KEYS);
    init_rgb(rgb_buffer, NUM_KEYS * 24 + 1);
    
    tusb_init();

    for (int i = 0; i < NUM_KEYS; i++) {
        hall_key_init(&keys[i]);
    }

    key_config_t config = {
        .actuation_mm = 150,
        .rt_down_mm = 10,
        .rt_up_mm = 10,
        .deadzone_top = 20,
        .deadzone_bottom = 20
    };

    int current_cal_key = 0;
    bool cal_complete = false;

    while (1) {
        tud_task();

        if (!cal_complete) {
            discovery_state_t state = hall_key_discovery_tick(&keys[current_cal_key], adc_buffer[current_cal_key]);
            if (state == DISCOVERY_DONE) {
                current_cal_key++;
                if (current_cal_key >= NUM_KEYS) cal_complete = true;
            }

            for (int i = 0; i < NUM_KEYS; i++) {
                if (i < current_cal_key) {
                    set_led(i, 0, 255, 0); // Green
                } else if (i == current_cal_key) {
                    if (keys[i].discovery_state == DISCOVERY_WAIT_RELEASE)
                        set_led(i, 255, 255, 0); // Yellow
                    else
                        set_led(i, 0, 0, 255); // Blue
                } else {
                    set_led(i, 0, 0, 0); // Off
                }
            }
            update_rgb(NUM_KEYS * 24 + 1);
        } else {
            if (tud_hid_ready()) {
                uint8_t keycode[6] = {0};
                uint8_t modifier = 0;
                int count = 0;

                for (int i = 0; i < NUM_KEYS; i++) {
                    if (hall_key_tick(&keys[i], adc_buffer[i], &config)) {
                        uint8_t usb_code = KEY_MAP[i];
                        if (usb_code >= 0xE0 && usb_code <= 0xE7) {
                            modifier |= (1 << (usb_code - 0xE0));
                        } else if (count < 6) {
                            keycode[count++] = usb_code;
                        }
                    }
                }
                tud_hid_keyboard_report(0, modifier, keycode);
            }
        }
    }
}

// TinyUSB callbacks
void tud_hid_set_report_cb(uint8_t itf, uint8_t report_id, hid_report_type_t report_type, uint8_t const* buffer, uint16_t bufsize) {}
uint16_t tud_hid_get_report_cb(uint8_t itf, uint8_t report_id, hid_report_type_t report_type, uint8_t* buffer, uint16_t reqlen) { return 0; }
