ENTRY(_start)

SECTIONS {
    . = 0x100000;
    
    .text : ALIGN(16) {
        *(.text._start)  /* _startを最初に配置 */
        *(.text .text.*)
    }
    
    .rodata : ALIGN(16) {
        *(.rodata .rodata.*)
    }
    
    .data : ALIGN(16) {
        *(.data .data.*)
    }
    
    .bss : ALIGN(16) {
        *(.bss .bss.*)
        *(COMMON)
    }
    
    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}