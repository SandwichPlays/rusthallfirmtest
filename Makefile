CC = arm-none-eabi-gcc
OBJCOPY = arm-none-eabi-objcopy
CFLAGS = -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 -O2 -g -Wall -Iinc
LDFLAGS = -Tlinker_script.ld -nostartfiles -Wl,--gc-sections

SRC = src/main.c src/hw.c src/he_logic.c src/startup.c
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
