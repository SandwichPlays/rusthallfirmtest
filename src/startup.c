#include <stdint.h>

extern uint32_t _estack;
extern uint32_t _sdata, _edata, _etext;
extern uint32_t _sbss, _ebss;

int main(void);

void Reset_Handler(void) {
    // Copy .data from FLASH to RAM
    uint32_t *src = &_etext;
    uint32_t *dst = &_sdata;
    while (dst < &_edata) *dst++ = *src++;

    // Zero .bss
    dst = &_sbss;
    while (dst < &_ebss) *dst++ = 0;

    main();
    while (1);
}

volatile uint32_t ms_ticks = 0;

void SysTick_Handler(void) {
    ms_ticks++;
}

uint32_t tusb_time_millis_api(void) {
    return ms_ticks;
}

// Minimal vector table
__attribute__((section(".vectors")))
void (*const vector_table[])(void) = {
    (void (*)(void))&_estack,
    Reset_Handler,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    SysTick_Handler // SysTick at index 15
};
