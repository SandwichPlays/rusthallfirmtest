#include "at32f405.h"

void init_clocks(void) {
    // 6 flash wait cycles for 216MHz
    ((volatile uint32_t *)FLASH_BASE)[0] = (((volatile uint32_t *)FLASH_BASE)[0] & ~0x07) | 0x06;

    CRM->CTRL |= (1 << 16); // HEXTEN
    while (!(CRM->CTRL & (1 << 17))); // HEXTSTBL

    // HEXT source, PLL_MS=1, PLL_NS=54, PLL_FR=1 => (8/1)*54/2 = 216MHz
    CRM->PLLCFG = (1 << 22) | (1 << 0) | (54 << 8) | (1 << 16);
    CRM->CTRL |= (1 << 24); // PLLEN
    while (!(CRM->CTRL & (1 << 25))); // PLLSTBL

    // AHB div 1, APB1 div 4, APB2 div 2, SCLKSEL PLL
    CRM->CFG = (0 << 4) | (4 << 10) | (3 << 13) | (2 << 0);
}

void init_adc_dma(uint16_t *buffer, uint16_t len) {
    CRM->AHBEN1 |= (1 << 0) | (1 << 17); // GPIOA, DMA1
    CRM->APB2EN |= (1 << 9); // ADC1

    ADC1->CTRL1 |= (1 << 8); // SQEN
    ADC1->CTRL2 |= (1 << 8); // OCDMAEN

    ADC1->OSQ1 = (3 << 20); // 4 conversions (testing) - extend for 64 keys later
    ADC1->OSQ3 = (0 << 0) | (1 << 5) | (2 << 10) | (3 << 15);

    DMA1->CH[0].PADDR = (uint32_t)&ADC1->ODATA;
    DMA1->CH[0].MADDR = (uint32_t)buffer;
    DMA1->CH[0].DTCNT = len;
    // Circular, MINCM, 16-bit, P2M
    DMA1->CH[0].CTRL = (1 << 5) | (1 << 7) | (1 << 8) | (1 << 10) | (1 << 0);

    ADC1->CTRL2 |= (1 << 22); // OCSWTRG
}

void init_rgb(uint16_t *buffer, uint16_t len) {
    CRM->AHBEN1 |= (1 << 0); // GPIOA
    CRM->APB2EN |= (1 << 0); // TMR1

    // PA8 MUX to AF1 (TMR1_CH1)
    GPIOA->MUXH = (GPIOA->MUXH & ~0x0F) | 0x01;
    GPIOA->CFGR = (GPIOA->CFGR & ~(0x03 << 16)) | (0x02 << 16);

    TMR1->PR = 269; 
    TMR1->DIV = 0;
    TMR1->CM1_OUTPUT = (6 << 4); // PWM1 mode
    TMR1->IDEN |= (1 << 8); // C1DEN

    DMA1->CH[1].PADDR = (uint32_t)&TMR1->C1DT;
    DMA1->CH[1].MADDR = (uint32_t)buffer;
    DMA1->CH[1].DTCNT = len;
    // MINCM, 16-bit, M2P
    DMA1->CH[1].CTRL = (1 << 7) | (1 << 10) | (1 << 8) | (1 << 4) | (0 << 0);

    TMR1->CTRL1 |= (1 << 0) | (1 << 7); // TMREN, PRBEN
    TMR1->BRK |= (1 << 15); // OEN
}

void update_rgb(uint16_t len) {
    DMA1->CH[1].CTRL &= ~(1 << 0);
    DMA1->CH[1].DTCNT = len;
    DMA1->CH[1].CTRL |= (1 << 0);
}
