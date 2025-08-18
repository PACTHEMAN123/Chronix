use core::ops::{Deref, DerefMut};

use crate::{irq::IrqCtrlHal, println};

struct RegRW {
    mmio_vbase: usize
}

impl RegRW {
    pub const fn new(mmio_vbase: usize) -> Self {
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

struct IntRegs {
    regs: RegRW
}

impl IntRegs {
    const INTISR: usize = 0x0;
    const INTEN: usize = 0x4;
    const INTENSET: usize = 0x8;
    const INTENCLR: usize = 0xc;
    const INTPOL: usize = 0x10;
    const INTEDGE: usize = 0x14;
    const INTBOUNCE: usize = 0x18;
    const INTAUTO: usize = 0x1c;

    pub const fn new(mmio_vbase: usize) -> Self {
        Self { regs: RegRW::new(mmio_vbase) }
    }
}

impl Deref for IntRegs {
    type Target = RegRW;

    fn deref(&self) -> &Self::Target {
        &self.regs
    }
}

impl DerefMut for IntRegs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.regs
    }
}

const INT_REGS: [IntRegs; 2] = [IntRegs::new(0x800000001fe01420), IntRegs::new(0x800000001fe01460)];

struct CoreRegs {
    regs: RegRW
}

impl CoreRegs {
    const CORE_IPISR: usize = 0x0;
    const CORE_IPIEN: usize = 0x4;
    const CORE_IPISET: usize = 0x8;
    const CORE_IPICLR: usize = 0xc;
    const CORE_BUF0: usize = 0x20;
    const CORE_BUF1: usize = 0x28;
    const CORE_BUF2: usize = 0x30;
    const CORE_BUF3: usize = 0x38;
    const CORE_INTISR0: usize = 0x40;
    const CORE_INTISR1: usize = 0x48;
    
    pub const fn new(mmio_vbase: usize) -> Self {
        Self { regs: RegRW::new(mmio_vbase) }
    }
}

impl Deref for CoreRegs {
    type Target = RegRW;

    fn deref(&self) -> &Self::Target {
        &self.regs
    }
}

impl DerefMut for CoreRegs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.regs
    }
}

const CORE_REGS: [CoreRegs; 2] = [CoreRegs::new(0x800000001fe01000), CoreRegs::new(0x800000001fe01100)];

struct IrqEntryRegs {
    regs: RegRW
}

impl IrqEntryRegs {
    const UART00_03: usize = 0x0;
    const UART04_07: usize = 0x1;
    const UART08_11: usize = 0x2;
    const DCI: usize = 0x3; // 专用通信接口 (Dedicated communication interface)
    const HDA_INT: usize = 0x4;
    const I2S_INT: usize = 0x5;
    const RESERVED0: usize = 0x6;
    const THSENS_INT: usize = 0x7;
    const TOY_TICK: usize = 0x8;
    const RTC_TICK: usize = 0x9;
    const CAM: usize = 0xa;
    const RESERVED1: usize = 0xb;
    const GMAC0_SDB_INT: usize = 0xc;
    const GMAC0_PMT_INT: usize = 0xd;
    const GMAC1_SDB_INT: usize = 0xe;
    const GMAC1_PMT_INT: usize = 0xf;
    const CAN0_INT: usize = 0x10;
    const CAN1_INT: usize = 0x11;
    const SPI_INT: usize = 0x12;
    const SATA_INT: usize = 0x13;
    const NAND_INT: usize = 0x14;
    const HPET_INT: usize = 0x15;
    const I2C_INT0: usize = 0x16;
    const I2C_INT1: usize = 0x17;
    const PWM_INT0: usize = 0x18;
    const PWM_INT1: usize = 0x19;
    const PWM_INT2: usize = 0x1a;
    const PWM_INT3: usize = 0x1b;
    const DC_INT: usize = 0x1c;
    const GPU_INT: usize = 0x1d;
    const VPU: usize = 0x1e;
    const SDIO_INT: usize = 0x1f;
    const PCIE0_INT0: usize = 0x40;
    const PCIE0_INT1: usize = 0x41;
    const PCIE0_INT2: usize = 0x42;
    const PCIE0_INT3: usize = 0x43;
    const PCIE1_INT0: usize = 0x44;
    const PCIE1_INT1: usize = 0x45;
    const HPET1_INT: usize = 0x46;
    const HPET2_INT: usize = 0x47;
    const TOY_INT0: usize = 0x48;
    const TOY_INT1: usize = 0x49;
    const TOY_INT2: usize = 0x4a;
    const TOY_INT3: usize = 0x4b;
    const DMA_INT0: usize = 0x4c;
    const DMA_INT1: usize = 0x4d;
    const DMA_INT2: usize = 0x4e;
    const DMA_INT3: usize = 0x4f;
    const DMA_INT4: usize = 0x50;
    const OTG_INT: usize = 0x51;
    const EHCI_INT: usize = 0x52;
    const OHCI_INT: usize = 0x53;
    const RTC_INT0: usize = 0x54;
    const RTC_INT1: usize = 0x55;
    const RTC_INT3: usize = 0x56;
    const RSA_INT: usize = 0x57;
    const AES_INT: usize = 0x58;
    const DES_INT: usize = 0x59;
    const GPIO_INT_LO: usize = 0x5a;
    const GPIO_INT_HI: usize = 0x5b;
    const GPIO_INT0: usize = 0x5c;
    const GPIO_INT1: usize = 0x5d;
    const GPIO_INT2: usize = 0x5e;
    const GPIO_INT3: usize = 0x5f;

    const ENTRIES: [usize; 64] = [
        Self::UART00_03,
        Self::UART04_07,
        Self::UART08_11,
        Self::DCI,
        Self::HDA_INT,
        Self::I2S_INT,
        Self::RESERVED0,
        Self::THSENS_INT,
        Self::TOY_TICK,
        Self::RTC_TICK,
        Self::CAM,
        Self::RESERVED1,
        Self::GMAC0_SDB_INT,
        Self::GMAC0_PMT_INT,
        Self::GMAC1_SDB_INT,
        Self::GMAC1_PMT_INT,
        Self::CAN0_INT,
        Self::CAN1_INT,
        Self::SPI_INT,
        Self::SATA_INT,
        Self::NAND_INT,
        Self::HPET_INT,
        Self::I2C_INT0,
        Self::I2C_INT1,
        Self::PWM_INT0,
        Self::PWM_INT1,
        Self::PWM_INT2,
        Self::PWM_INT3,
        Self::DC_INT,
        Self::GPU_INT,
        Self::VPU,
        Self::SDIO_INT,
        Self::PCIE0_INT0,
        Self::PCIE0_INT1,
        Self::PCIE0_INT2,
        Self::PCIE0_INT3,
        Self::PCIE1_INT0,
        Self::PCIE1_INT1,
        Self::HPET1_INT,
        Self::HPET2_INT,
        Self::TOY_INT0,
        Self::TOY_INT1,
        Self::TOY_INT2,
        Self::TOY_INT3,
        Self::DMA_INT0,
        Self::DMA_INT1,
        Self::DMA_INT2,
        Self::DMA_INT3,
        Self::DMA_INT4,
        Self::OTG_INT,
        Self::EHCI_INT,
        Self::OHCI_INT,
        Self::RTC_INT0,
        Self::RTC_INT1,
        Self::RTC_INT3,
        Self::RSA_INT,
        Self::AES_INT,
        Self::DES_INT,
        Self::GPIO_INT_LO,
        Self::GPIO_INT_HI,
        Self::GPIO_INT0,
        Self::GPIO_INT1,
        Self::GPIO_INT2,
        Self::GPIO_INT3,
    ];

    pub const fn new(mmio_vbase: usize) -> Self {
        Self { regs: RegRW::new(mmio_vbase) }
    }
}

impl Deref for IrqEntryRegs {
    type Target = RegRW;

    fn deref(&self) -> &Self::Target {
        &self.regs
    }
}

impl DerefMut for IrqEntryRegs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.regs
    }
}

const IRQ_ENTRY_REGS: IrqEntryRegs = IrqEntryRegs::new(0x800000001fe01400); 

pub struct IrqCtrl;

impl IrqCtrlHal for IrqCtrl {
    fn from_dt(device_tree: &fdt::Fdt, mmio: impl crate::mapper::MmioMapperHal) -> Option<Self> where Self: Sized {
        for no in 0..64 {
            IRQ_ENTRY_REGS.write_b(IrqEntryRegs::ENTRIES[no], 0x11);
        }
        INT_REGS[0].write_w(IntRegs::INTENCLR,!0u32);
        INT_REGS[0].write_w(IntRegs::INTPOL,0u32);
        INT_REGS[0].write_w(IntRegs::INTEDGE,0u32);
        INT_REGS[0].write_w(IntRegs::INTBOUNCE,0u32);
        INT_REGS[0].write_w(IntRegs::INTAUTO,0u32);
        INT_REGS[1].write_w(IntRegs::INTENCLR, !0u32);
        INT_REGS[1].write_w(IntRegs::INTPOL,0u32);
        INT_REGS[1].write_w(IntRegs::INTEDGE,0u32);
        INT_REGS[1].write_w(IntRegs::INTBOUNCE,0u32);
        INT_REGS[1].write_w(IntRegs::INTAUTO,0u32);
        Some(IrqCtrl)
    }

    fn enable_irq(&self, no: usize, _: usize) {
        IRQ_ENTRY_REGS.write_b(IrqEntryRegs::ENTRIES[no], 0x11);
        if no < 32 {
            INT_REGS[0].write_w(IntRegs::INTENSET, 1u32 << no);
        } else {
            INT_REGS[1].write_w(IntRegs::INTENSET, 1u32 << (no - 32));
        }
    }

    fn disable_irq(&self, no: usize, _: usize) {
        if no < 32 {
            INT_REGS[0].write_w(IntRegs::INTENCLR, 1u32 << no);
        } else {
            INT_REGS[1].write_w(IntRegs::INTENCLR, 1u32 << (no - 32));
        }
    }

    fn claim_irq(&self, _: usize) -> Option<usize> {
        let high32 = CORE_REGS[0].read_w(CoreRegs::CORE_INTISR1);
        let no = 32 - high32.leading_zeros();
        if no != 0 {
            return Some(no as usize + 31);
        }
        let low32 = CORE_REGS[0].read_w(CoreRegs::CORE_INTISR0);
        let no = 32 - low32.leading_zeros();
        if no != 0 {
            return Some(no as usize - 1);
        }
        return None;
    }

    fn complete_irq(&self, no: usize, _: usize) {
        if no < 32 {
            INT_REGS[0].write_w(IntRegs::INTENCLR, 1u32 << no);
            INT_REGS[0].write_w(IntRegs::INTENSET, 1u32 << no);
        } else {
            INT_REGS[1].write_w(IntRegs::INTENCLR, 1u32 << (no - 32));
            INT_REGS[1].write_w(IntRegs::INTENSET, 1u32 << (no - 32));
        }
    }
}
