#include "at32f405.h"

void init_clocks(void) {
    ((volatile uint32_t *)FLASH_BASE)[0] = (((volatile uint32_t *)FLASH_BASE)[0] & ~0x07) | 0x06;
    CRM->CTRL |= (1 << 16);
    while (!(CRM->CTRL & (1 << 17)));
    CRM->PLLCFG = (1 << 22) | (1 << 0) | (54 << 8) | (1 << 16);
    CRM->CTRL |= (1 << 24);
    while (!(CRM->CTRL & (1 << 25)));
    CRM->CFG = (0 << 4) | (4 << 10) | (3 << 13) | (2 << 0);
}

void init_adc_dma(uint16_t *buffer, uint16_t len) {
    CRM->AHBEN1 |= (1 << 0) | (1 << 17); 
    CRM->APB2EN |= (1 << 9);

    // PA0-PA3: Mux Address, PA4-PA7: ADC Inputs
    GPIOA->CTRL = (GPIOA->CTRL & ~0xFFFFFF) | 0x22221111; // PA0-3: Out, PA4-7: Analog

    ADC1->CTRL1 |= (1 << 8); 
    ADC1->CTRL2 |= (1 << 8); 

    ADC1->OSQ1 = (3 << 20); // Scan 4 mux outputs at once
    ADC1->OSQ3 = (4 << 0) | (5 << 5) | (6 << 10) | (7 << 15);

    DMA1->CH[0].PADDR = (uint32_t)&ADC1->ODATA;
    DMA1->CH[0].MADDR = (uint32_t)buffer;
    DMA1->CH[0].DTCNT = 4; // 4 channels per mux step
    DMA1->CH[0].CTRL = (1 << 7) | (1 << 8) | (1 << 10) | (1 << 0);
}

// Call this at 8kHz to scan one mux step (4 keys)
// Complete scan takes 16 steps (2ms @ 8kHz, or faster if called in loop)
void scan_mux_step(int step) {
    GPIOA->ODR = (GPIOA->ODR & ~0x0F) | (step & 0x0F);
    ADC1->CTRL2 |= (1 << 22); // Trigger conversion
}

void init_rgb(uint16_t *buffer, uint16_t len) {
    CRM->AHBEN1 |= (1 << 0);
    CRM->APB2EN |= (1 << 0);
    GPIOA->MUXH = (GPIOA->MUXH & ~0x0F) | 0x01;
    GPIOA->CFGR = (GPIOA->CFGR & ~(0x03 << 16)) | (0x02 << 16);
    TMR1->PR = 269; 
    TMR1->DIV = 0;
    TMR1->CM1_OUTPUT = (6 << 4);
    TMR1->IDEN |= (1 << 8);
    DMA1->CH[1].PADDR = (uint32_t)&TMR1->C1DT;
    DMA1->CH[1].MADDR = (uint32_t)buffer;
    DMA1->CH[1].DTCNT = len;
    DMA1->CH[1].CTRL = (1 << 7) | (1 << 10) | (1 << 8) | (1 << 4) | (0 << 0);
    TMR1->CTRL1 |= (1 << 0) | (1 << 7);
    TMR1->BRK |= (1 << 15);
}

void update_rgb(uint16_t len) {
    DMA1->CH[1].CTRL &= ~(1 << 0);
    DMA1->CH[1].DTCNT = len;
    DMA1->CH[1].CTRL |= (1 << 0);
}
