ENTRY(_start)

PHDRS {
    code PT_LOAD FLAGS(5);  /* R + E */
    data PT_LOAD FLAGS(6);  /* R + W */
}


SECTIONS {
    . = 0x10000;

    .text : {
        KEEP(*(.text._start))
        *(.text .text.*)
    } :code

    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } :code

    . = ALIGN(0x1000);

    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.note .note.*)
        *(.riscv.attributes)
    }
}
