pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
    // (0x1fe0_01e0, 0x00_0100), // UART in virt machine
];

pub const MAX_PROCESSORS: usize = 4;

core::arch::global_asm!{
    "
    .section .rodata
    .global _dtb_start
    .global _dtb_end
    _dtb_start:
        .incbin \"hal/src/board/loongarch64-qemu.dtb\"
    _end_end:
    "
}
