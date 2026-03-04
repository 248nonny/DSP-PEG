ENTRY(_start)

BASE_ADDR = 0x11000000;
MAX_ADDR = 0x1b47efff;

MEMORY {
  IMG (rwx) : ORIGIN = BASE_ADDR, LENGTH = (MAX_ADDR - BASE_ADDR + 1)
}

SECTIONS
{
    . = ORIGIN(IMG);

    .text :
    {
        KEEP(*(.text._start))
        *(.text*)
    } > IMG

    
    .rodata ALIGN(4096) : { *(.rodata*) } > IMG
    .data   ALIGN(4096) : { *(.data*)   } > IMG

    .bss ALIGN(4096) : {
      __bss_start = .;
      *(.bss*)
      __bss_end = .;
    } > IMG    

    /* Put stacks near the very top of allocated memory region */
    . = (MAX_ADDR & 0xFFF00000) - (12 * 1024 * 1024);
    .stack1 (NOLOAD) : { . = . + 1024 * 1024; _stack_start_1 = .; } > IMG
    .stack2 (NOLOAD) : { . = . + 1024 * 1024; _stack_start_2 = .; } > IMG
    .stack3 (NOLOAD) : { . = . + 1024 * 1024; _stack_start_3 = .; } > IMG

    .image_end_marker : { __image_end = .; } > IMG
}
