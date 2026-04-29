MEMORY
{
  FLASH (rx) : ORIGIN = 0x08010000, LENGTH = 192K
  RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 96K
}

ENTRY(Reset_Handler)

SECTIONS
{
  .text :
  {
    KEEP(*(.vectors))
    *(.text*)
    *(.rodata*)
    _etext = .;
  } > FLASH

  .data : AT(_etext)
  {
    _sdata = .;
    *(.data*)
    _edata = .;
  } > RAM

  .bss :
  {
    _sbss = .;
    *(.bss*)
    *(COMMON)
    _ebss = .;
  } > RAM

  _estack = ORIGIN(RAM) + LENGTH(RAM);

  /DISCARD/ :
  {
    *(.ARM.exidx*)
  }
}
