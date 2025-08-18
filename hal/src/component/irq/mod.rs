#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod la2k1000;

#[cfg(target_arch = "loongarch64")]
pub use la2k1000::*;

pub trait IrqCtrlHal {
    fn from_dt(device_tree: &fdt::Fdt, mmio: impl crate::mapper::MmioMapperHal) -> Option<Self> where Self: Sized;
    fn enable_irq(&self, no: usize, ctx_id: usize);
    fn disable_irq(&self, no: usize, ctx_id: usize);
    fn claim_irq(&self, ctx_id: usize) -> Option<usize>;
    fn complete_irq(&self, no: usize, ctx_id: usize);
}