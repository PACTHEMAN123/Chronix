#[allow(unused)]
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];

pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;

pub const MAX_PROCESSORS: usize = 4;