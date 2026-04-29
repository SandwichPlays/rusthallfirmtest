#include "at32f405.h"
#include "he_logic.h"
#include "tusb.h"

// Hardware hooks
void init_clocks(void);
void init_adc_dma(uint16_t *buffer, uint16_t len);
void scan_mux_step(int step);
void init_rgb(uint16_t *buffer, uint16_t len);
void update_rgb(uint16_t len);
void save_calibration(hall_key_t *keys);
void load_calibration(hall_key_t *keys);

static uint16_t adc_raw[4]; // 4 channels per step
static uint16_t adc_history[NUM_KEYS];
static uint16_t rgb_buffer[NUM_KEYS * 24 + 1];
static hall_key_t keys[NUM_KEYS];

typedef struct TU_ATTR_PACKED {
    uint8_t modifier;
    uint8_t reserved;
    uint8_t key_mask[16];
} hid_nkro_report_t;

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
    
    // Configure SysTick for 1ms interrupts
    *((volatile uint32_t *)0xE000E014) = 216000 - 1; // Load value for 1ms @ 216MHz
    *((volatile uint32_t *)0xE000E018) = 0;          // Current value
    *((volatile uint32_t *)0xE000E010) = 0x07;       // Enable, Interrupt, Core clock
    
    init_adc_dma(adc_raw, 4);
    init_rgb(rgb_buffer, NUM_KEYS * 24 + 1);
    tusb_init();

    for (int i = 0; i < NUM_KEYS; i++) hall_key_init(&keys[i]);
    load_calibration(keys);

    key_config_t config = {150, 10, 10, 20, 20};
    int current_cal_key = 0;
    bool cal_complete = false;

    // Check if loaded calibration is valid
    if (keys[0].discovery_state == DISCOVERY_DONE) {
        cal_complete = true;
    }
    int mux_step = 0;

    while (1) {
        tud_task();

        // Scan 4 keys per loop iteration
        scan_mux_step(mux_step);
        for (int i = 0; i < 4; i++) {
            adc_history[mux_step * 4 + i] = adc_raw[i];
        }
        mux_step = (mux_step + 1) % 16;

        if (!cal_complete) {
            discovery_state_t state = hall_key_discovery_tick(&keys[current_cal_key], adc_history[current_cal_key]);
            if (state == DISCOVERY_DONE) {
                current_cal_key++;
                if (current_cal_key >= NUM_KEYS) {
                    cal_complete = true;
                    save_calibration(keys);
                }
            }

            for (int i = 0; i < NUM_KEYS; i++) {
                if (i < current_cal_key) set_led(i, 0, 255, 0);
                else if (i == current_cal_key) {
                    if (keys[i].discovery_state == DISCOVERY_WAIT_RELEASE) set_led(i, 255, 255, 0);
                    else set_led(i, 0, 0, 255);
                } else set_led(i, 0, 0, 0);
            }
            update_rgb(NUM_KEYS * 24 + 1);
        } else {
            if (tud_hid_ready()) {
                hid_nkro_report_t report = {0};
                for (int i = 0; i < NUM_KEYS; i++) {
                    if (hall_key_tick(&keys[i], adc_history[i], &config)) {
                        uint8_t usb_code = KEY_MAP[i];
                        if (usb_code >= 0xE0 && usb_code <= 0xE7) {
                            report.modifier |= (1 << (usb_code - 0xE0));
                        } else if (usb_code < 128) {
                            report.key_mask[usb_code / 8] |= (1 << (usb_code % 8));
                        }
                    }
                }
                tud_hid_report(1, &report, sizeof(report));
            }
        }
    }
}

void tud_hid_set_report_cb(uint8_t itf, uint8_t report_id, hid_report_type_t report_type, uint8_t const* buffer, uint16_t bufsize) {}
uint16_t tud_hid_get_report_cb(uint8_t itf, uint8_t report_id, hid_report_type_t report_type, uint8_t* buffer, uint16_t reqlen) { return 0; }

void OTG_HS_IRQHandler(void) {
    tud_int_handler(0);
}
