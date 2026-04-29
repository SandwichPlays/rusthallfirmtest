#ifndef TUSB_CONFIG_H
#define TUSB_CONFIG_H

#define CFG_TUSB_MCU                OPT_MCU_STM32F4
#define CFG_TUSB_RHPORT0_MODE      (OPT_MODE_DEVICE | OPT_MODE_HIGH_SPEED)
#define CFG_TUSB_RHPORT0_HS         1

#define CFG_TUD_ENDPOINT0_SIZE      64
#define CFG_TUD_HID                 1
#define CFG_TUD_HID_EP_BUFSIZE      64

#endif
