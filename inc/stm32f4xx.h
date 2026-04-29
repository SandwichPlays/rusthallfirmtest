#ifndef STM32F4XX_H
#define STM32F4XX_H

#include "at32f405.h"

// Map AT32 registers to what TinyUSB expects for STM32
#define USB_OTG_HS_PERIPH_BASE  0x40040000
#define USB_OTG_HS              ((void*)USB_OTG_HS_PERIPH_BASE)

// TinyUSB stm32 dcd uses these IRQ names
#define OTG_HS_IRQn             77

#endif
