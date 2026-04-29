#ifndef AT32F405_H
#define AT32F405_H

#include <stdint.h>

#define CRM_BASE        0x40021000
#define FLASH_BASE      0x40023C00
#define GPIOA_BASE      0x40010800
#define ADC1_BASE       0x40012400
#define DMA1_BASE       0x40026000
#define TMR1_BASE       0x40010000
#define USBHS_BASE      0x40040000

typedef struct {
  volatile uint32_t CTRL;
  volatile uint32_t CFG;
  volatile uint32_t INTR;
  volatile uint32_t APB2RST;
  volatile uint32_t APB1RST;
  volatile uint32_t AHBEN;
  volatile uint32_t APB2EN;
  volatile uint32_t APB1EN;
  volatile uint32_t AHBEN1;
  volatile uint32_t AHBEN2;
  volatile uint32_t AHBEN3;
  volatile uint32_t APB2EN1;
  volatile uint32_t APB1EN1;
  volatile uint32_t AHBRST1;
  volatile uint32_t AHBRST2;
  volatile uint32_t AHBRST3;
  volatile uint32_t APB2RST1;
  volatile uint32_t APB1RST1;
  volatile uint32_t PLLCFG;
  volatile uint32_t CLKOUT1;
  volatile uint32_t CLKOUT2;
  volatile uint32_t OTGHS;
} CRM_Type;

typedef struct {
  volatile uint32_t CTRL;
  volatile uint32_t CFGR;
  volatile uint32_t ODR;
  volatile uint32_t BSRR;
  volatile uint32_t BRR;
  volatile uint32_t SCR;
  volatile uint32_t MUXL;
  volatile uint32_t MUXH;
} GPIO_Type;

typedef struct {
  volatile uint32_t CTRL;
  volatile uint32_t PADDR;
  volatile uint32_t MADDR;
  volatile uint32_t DTCNT;
} DMA_Channel_Type;

typedef struct {
  volatile uint32_t STS;
  DMA_Channel_Type CH[7];
} DMA_Type;

typedef struct {
  volatile uint32_t STS;
  volatile uint32_t CTRL1;
  volatile uint32_t CTRL2;
  volatile uint32_t SPT1;
  volatile uint32_t SPT2;
  volatile uint32_t HT;
  volatile uint32_t LT;
  volatile uint32_t OSQ1;
  volatile uint32_t OSQ2;
  volatile uint32_t OSQ3;
  volatile uint32_t ISQ;
  volatile uint32_t IDATA[4];
  volatile uint32_t ODATA;
} ADC_Type;

typedef struct {
  volatile uint32_t CTRL1;
  volatile uint32_t CTRL2;
  volatile uint32_t SMCTRL;
  volatile uint32_t IDEN;
  volatile uint32_t STS;
  volatile uint32_t SWEVT;
  volatile uint32_t CM1_OUTPUT;
  volatile uint32_t CM2_OUTPUT;
  volatile uint32_t CCTRL;
  volatile uint32_t CNT;
  volatile uint32_t DIV;
  volatile uint32_t PR;
  volatile uint32_t RPR;
  volatile uint32_t C1DT;
  volatile uint32_t C2DT;
  volatile uint32_t C3DT;
  volatile uint32_t C4DT;
  volatile uint32_t BRK;
  volatile uint32_t DMACTRL;
  volatile uint32_t DMAADDR;
} TMR_Type;

#define CRM     ((CRM_Type *) CRM_BASE)
#define GPIOA   ((GPIO_Type *) GPIOA_BASE)
#define ADC1    ((ADC_Type *) ADC1_BASE)
#define DMA1    ((DMA_Type *) DMA1_BASE)
#define TMR1    ((TMR_Type *) TMR1_BASE)

#endif
