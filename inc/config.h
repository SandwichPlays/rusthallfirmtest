#ifndef CONFIG_H
#define CONFIG_H

#include <stdint.h>

// --- HARDWARE SETUP ---
#define NUM_KEYS            4       // Set to 4 for your current test, 64 for full board
#define USE_MULTIPLEXERS    0       // 1 = Enabled, 0 = Direct wired (for your 4-key test)

// Pin Mappings
#define RGB_PIN             8       // PA8 (Must be a TMR1_CH1 compatible pin)
#define ADC_START_PIN       4       // PA4, PA5, PA6, PA7...

// --- SENSOR SETTINGS (centi-mm: 150 = 1.5mm) ---
#define DEFAULT_ACTUATION   150     
#define DEFAULT_RT_DOWN     10      
#define DEFAULT_RT_UP       10      
#define DEFAULT_TOP_DZ      20      
#define DEFAULT_BOTTOM_DZ   20      

// --- KEY MAP ---
// Define your layout here. Standard HID scan codes.
#define MY_KEY_MAP { \
    0x04, 0x05, 0x06, 0x07, \
}

#endif
