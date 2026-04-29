CC = arm-none-eabi-gcc
OBJCOPY = arm-none-eabi-objcopy
TINYUSB_ROOT = tinyusb
CFLAGS = -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 -O2 -g -Wall -Iinc -I$(TINYUSB_ROOT)/src -DSTM32F405xx -fno-exceptions -fno-unwind-tables
LDFLAGS = -Tlinker_script.ld -nostartfiles -Wl,--gc-sections

SRC = src/main.c src/hw.c src/he_logic.c src/startup.c src/usb_descriptors.c src/flash.c
# Add TinyUSB sources
SRC += $(TINYUSB_ROOT)/src/tusb.c \
       $(TINYUSB_ROOT)/src/common/tusb_fifo.c \
       $(TINYUSB_ROOT)/src/device/usbd.c \
       $(TINYUSB_ROOT)/src/device/usbd_control.c \
       $(TINYUSB_ROOT)/src/class/hid/hid_device.c \
       $(TINYUSB_ROOT)/src/portable/synopsys/dwc2/dcd_dwc2.c \
       $(TINYUSB_ROOT)/src/portable/synopsys/dwc2/dwc2_common.c
OBJ = $(SRC:.c=.o)

all: firmware.bin

firmware.elf: $(OBJ)
	$(CC) $(CFLAGS) $(OBJ) $(LDFLAGS) -o $@

firmware.bin: firmware.elf
	$(OBJCOPY) -O binary $< $@

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJ) firmware.elf firmware.bin
