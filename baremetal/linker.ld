ENTRY(_start)

BASE_ADDR = 0x10100000;
/* BASE_ADDR = 0x80000; */
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

    .stack1 ALIGN(4096) : { . = . + 1024 * 1024; _stack_start_1 = .; } > IMG
    .stack2 ALIGN(4096) : { . = . + 1024 * 1024; _stack_start_2 = .; } > IMG
    .stack3 ALIGN(4096) : { . = . + 1024 * 1024; _stack_start_3 = .; } > IMG
    .stack_extra ALIGN(4096) : { . = . + 1024 * 1024; } > IMG

    .image_end_marker : { __image_end = .; } > IMG
}
