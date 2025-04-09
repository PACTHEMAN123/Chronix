pub const MEMORY_END: usize = 0x8800_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_0000, 0x00_0100), // UART in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
    (0x1000_2000, 0x00_0200), // Virtio Net in virt machine
];

pub const MAX_PROCESSORS: usize = 4;

// uart related
// from qemu-system-riscv64 -virt device tree source
pub const UART_MMIO_BASE_PADDR: usize = 0x10000000;
pub const UART_CLK_FEQ: usize = 0x384000;
pub const UART_MMIO_SIZE: usize = 0x100;
pub const UART_IRQ_NUM: usize = 0x0a;
pub const UART_REG_IO_WIDTH: usize = 0x1;
pub const UART_REG_SHIFT: usize = 0x0;