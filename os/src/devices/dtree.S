    .section .rodata
    .global _dtb_start
    .global _dtb_end
_dtb_start:
    .incbin "os/src/devices/dtree.dtb"
_end_end: