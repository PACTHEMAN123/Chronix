pub const MEMORY_END: usize = 0x9600_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
    (0x1fe0_01e0, 0x00_0100), // UART in virt machine
];

pub const MAX_PROCESSORS: usize = 4;

// uart related
// from qemu-system-loongarch64 -virt device tree source
pub const UART_MMIO_BASE_PADDR: usize = 0x1fe001e0;
pub const UART_CLK_FEQ: usize = 0x5f5e100;
pub const UART_MMIO_SIZE: usize = 0x100;
pub const UART_IRQ_NUM: usize = 0x02;
pub const UART_REG_IO_WIDTH: usize = 0x1;
pub const UART_REG_SHIFT: usize = 0x0;