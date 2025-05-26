/// Platform Interrupt Contorller
pub struct Platic {
    /// virtual address
    mmio_vbase: usize,
}

impl Platic {
    pub const INT_ID: usize = 0x000;
    pub const INT_MASK: usize = 0x020;
    pub const HTMSI_EN: usize = 0x040;
    pub const INTEDGE: usize = 0x060;
    pub const INTCLR: usize = 0x080;
    pub const AUTO_CTRL0: usize = 0x0c0;
    pub const AUTO_CTRL1: usize = 0x0e0;
    pub const ROUTE_ENTRY_0: usize = 0x100;
    pub const ROUTE_ENTRY_8: usize = 0x108;
    pub const ROUTE_ENTRY_16: usize = 0x110;
    pub const ROUTE_ENTRY_24: usize = 0x118;
    pub const ROUTE_ENTRY_32: usize = 0x120;
    pub const ROUTE_ENTRY_40: usize = 0x128;
    pub const ROUTE_ENTRY_48: usize = 0x130;
    pub const ROUTE_ENTRY_56: usize = 0x138;
    pub const HTMSI_VECTOR0: usize = 0x200;
    pub const HTMSI_VECTOR8: usize = 0x208;
    pub const HTMSI_VECTOR16: usize = 0x210;
    pub const HTMSI_VECTOR24: usize = 0x218;
    pub const HTMSI_VECTOR32: usize = 0x220;
    pub const HTMSI_VECTOR40: usize = 0x228;
    pub const HTMSI_VECTOR48: usize = 0x230;
    pub const HTMSI_VECTOR56: usize = 0x238;
    pub const INTISR_0: usize = 0x300;
    pub const INTISR_1: usize = 0x320;
    pub const INTIRR: usize = 0x380;
    pub const INTISR: usize = 0x3a0;
    pub const INT_POLARITY: usize = 0x3e0;

    pub fn new(mmio_vbase: usize) -> Self {
        Self { mmio_vbase }
    }

    pub fn write_b(&self, offset: usize, val: u8) {
        unsafe {
            ((self.mmio_vbase + offset) as *mut u8).write_volatile(val);
        }
    }

    pub fn write_h(&self, offset: usize, val: u16) {
        unsafe {
            ((self.mmio_vbase + offset) as *mut u16).write_volatile(val);
        }
    }

    pub fn write_w(&self, offset: usize, val: u32) {
        unsafe {
            ((self.mmio_vbase + offset) as *mut u32).write_volatile(val);
        }
    }

    pub fn read_b(&self, offset: usize) -> u8 {
        unsafe {
            ((self.mmio_vbase + offset) as *mut u8).read_volatile()
        }
    }

    pub fn read_h(&self, offset: usize) -> u16 {
        unsafe {
            ((self.mmio_vbase + offset) as *mut u16).read_volatile()
        }
    }

    pub fn read_w(&self, offset: usize) -> u32 {
        unsafe {
            ((self.mmio_vbase + offset) as *mut u32).read_volatile()
        }
    }
}  