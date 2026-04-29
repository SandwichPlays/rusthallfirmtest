#ifndef STM32F4XX_H
#define STM32F4XX_H

#include <stdint.h>
#include "at32f405.h"

// TinyUSB / CMSIS definitions
#define USB_OTG_HS_PERIPH_BASE      0x40040000
#define USB_OTG_HS                  ((void*)USB_OTG_HS_PERIPH_BASE)
#define OTG_HS_IRQn                 77

#define USB_OTG_HS_MAX_IN_ENDPOINTS 6
#define DFIFO_DEPTH_HS              512

typedef int IRQn_Type;

// CMSIS Functions
static inline void NVIC_EnableIRQ(IRQn_Type irq) {
    // AT32 uses standard NVIC
    *((volatile uint32_t *)(0xE000E100 + (irq / 32) * 4)) = (1 << (irq % 32));
}

static inline void NVIC_DisableIRQ(IRQn_Type irq) {
    *((volatile uint32_t *)(0xE000E180 + (irq / 32) * 4)) = (1 << (irq % 32));
}

#define __NOP() __asm volatile ("nop")

extern uint32_t SystemCoreClock;

#endif
