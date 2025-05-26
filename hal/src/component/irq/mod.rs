#[cfg(target_arch = "riscv64")]
mod riscv64;

use core::range::Range;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;

pub trait IrqCtrlHal {
    fn from_dt(device_tree: &fdt::Fdt, mmio: impl crate::mapper::MmioMapperHal) -> Option<Self> where Self: Sized;
    fn enable_irq(&self, no: usize);
    fn disable_irq(&self, no: usize);
    fn claim_irq(&self) -> Option<usize>;
    fn complete_irq(&self, no: usize);
}