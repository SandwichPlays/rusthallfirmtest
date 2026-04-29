#include "at32f405.h"
#include "he_logic.h"

#define FLASH_USER_START_ADDR   0x0803F800 // Last 2KB of 256KB Flash
#define FLASH_KEY               0x45670123
#define FLASH_KEY2              0xCDEF89AB

typedef struct {
    uint32_t magic;
    uint16_t baseline[NUM_KEYS];
    uint16_t max_travel[NUM_KEYS];
} flash_data_t;

#define FLASH_MAGIC 0xDEADBEEF

void flash_unlock(void) {
    if (((volatile uint32_t *)FLASH_BASE)[1] & (1 << 7)) { // FLASH_CTRL.LOCK
        ((volatile uint32_t *)FLASH_BASE)[4] = FLASH_KEY;  // FLASH_KEY
        ((volatile uint32_t *)FLASH_BASE)[4] = FLASH_KEY2;
    }
}

void flash_lock(void) {
    ((volatile uint32_t *)FLASH_BASE)[1] |= (1 << 7);
}

void save_calibration(hall_key_t *keys) {
    flash_unlock();
    
    // Erase page
    ((volatile uint32_t *)FLASH_BASE)[1] |= (1 << 1); // PER (Page Erase)
    ((volatile uint32_t *)FLASH_BASE)[5] = FLASH_USER_START_ADDR; // ADR
    ((volatile uint32_t *)FLASH_BASE)[1] |= (1 << 6); // STRT
    while (((volatile uint32_t *)FLASH_BASE)[3] & (1 << 0)); // BSY
    ((volatile uint32_t *)FLASH_BASE)[1] &= ~(1 << 1);

    // Program data
    flash_data_t data;
    data.magic = FLASH_MAGIC;
    for (int i = 0; i < NUM_KEYS; i++) {
        data.baseline[i] = keys[i].baseline;
        data.max_travel[i] = keys[i].max_travel;
    }

    uint16_t *src = (uint16_t *)&data;
    uint32_t *dst = (uint32_t *)FLASH_USER_START_ADDR;
    
    for (int i = 0; i < sizeof(flash_data_t) / 4; i++) {
        ((volatile uint32_t *)FLASH_BASE)[1] |= (1 << 0); // PG (Program)
        dst[i] = ((uint32_t *)src)[i];
        while (((volatile uint32_t *)FLASH_BASE)[3] & (1 << 0)); // BSY
    }
    
    ((volatile uint32_t *)FLASH_BASE)[1] &= ~(1 << 0);
    flash_lock();
}

void load_calibration(hall_key_t *keys) {
    flash_data_t *data = (flash_data_t *)FLASH_USER_START_ADDR;
    if (data->magic == FLASH_MAGIC) {
        for (int i = 0; i < NUM_KEYS; i++) {
            keys[i].baseline = data->baseline[i];
            keys[i].max_travel = data->max_travel[i];
            keys[i].discovery_state = DISCOVERY_DONE;
        }
    }
}
