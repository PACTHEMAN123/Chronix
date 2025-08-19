pub const MAX_PROCESSORS: usize = 2;

core::arch::global_asm!{
    "
    .section .rodata
    .global _dtb_start
    .global _dtb_end
    _dtb_start:
    .align 12
        .incbin \"hal/src/board/dtbs/ls2k1000la.dtb\"
    _end_end:
    "
}
