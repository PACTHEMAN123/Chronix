mod platic;

use fdt::Fdt;
pub use platic::*;

mod eiointc;

pub use eiointc::*;

use crate::{mapper::MmioMapperHal, println};

use super::IrqCtrlHal;

pub struct IrqCtrl {
    platic: Platic,
}

impl IrqCtrlHal for IrqCtrl {
    fn from_dt(root: &Fdt, mmio: impl MmioMapperHal) -> Option<Self> {
        let platic = root.find_compatible(&["loongson,pch-pic-1.0"])?;
        let cpu_num = root.find_all_nodes("/cpus/cpu").count();
        log::info!("[IrqCtrl::from_dt] cpu number: {cpu_num}");
        Eiointc::init(cpu_num);
        let platic_region = platic.reg()?.next()?;
        let start = platic_region.starting_address as usize;
        let size = platic_region.size?;
        let vregion = mmio.map_mmio_area(start..start+size);
        let platic = Platic::new(vregion.start);
        platic.write_w(Platic::INT_POLARITY, 0x0);
        platic.write_w(Platic::INT_POLARITY + 4, 0x0);
        platic.write_w(Platic::INTEDGE, 0x0);
        platic.write_w(Platic::INTEDGE + 4, 0x0);
        Some(Self { platic })
    }
    
    fn enable_irq(&self, no: usize) {
        Eiointc::enable_irq(no);
        if no < 32 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask &= !(1 << no);
            self.platic.write_w(Platic::INT_MASK, mask);
            self.platic.write_b(Platic::HTMSI_VECTOR0 + no, no as u8);
        } else if no < 64 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask &= !(1 << (no - 32));
            self.platic.write_w(Platic::INT_MASK, mask);
            self.platic.write_b(Platic::HTMSI_VECTOR32 + no, no as u8);
        } else {
            log::warn!("[IrqCtrl] irq_no > 64");
        }
    }
    
    fn disable_irq(&self, no: usize) {
        Eiointc::disable_irq(no);
        if no < 32 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask |= 1 << no;
            self.platic.write_w(Platic::INT_MASK, mask);
        } else if no < 64 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask |= 1 << (no - 32);
            self.platic.write_w(Platic::INT_MASK, mask);
        } else {
            log::warn!("[IrqCtrl] irq_no > 64");
        }
    }
    
    fn claim_irq(&self) -> Option<usize> {
        Eiointc::claim_irq()
    }
    
    fn complete_irq(&self, no: usize) {
        Eiointc::disable_irq(no);
    }
}

