use fdt::Fdt;
use hal::constant::{Constant, ConstantsHal};

pub struct PLIC {
    /// MMIO base address.
    pub mmio_base: usize,
    /// MMIO region size.
    pub mmio_size: usize,
}

// const PLIC_ADDR: usize = 0xc00_0000 + VIRT_RAM_OFFSET;

impl PLIC {
    pub fn new(mmio_base: usize, mmio_size: usize) -> PLIC {
        PLIC {
            mmio_base,
            mmio_size,
        }
    }

    pub fn enable_irq(&self, irq: usize, ctx_id: usize) {
        let plic = (self.mmio_base + Constant::KERNEL_ADDR_SPACE.start) as *mut plic::Plic;

        // Setup PLIC
        let src = PLICSrcWrapper::new(irq);
        let ctx = PLICCtxWrapper::new(ctx_id);

        unsafe { (*plic).set_threshold(ctx, 0) };
        unsafe { (*plic).enable(src, ctx) };
        unsafe { (*plic).set_priority(src, 6) };
    }

    /// Return the IRQ number of the highest priority pending interrupt
    pub fn claim_irq(&self, ctx_id: usize) -> Option<usize> {
        let plic = (self.mmio_base + Constant::KERNEL_ADDR_SPACE.start) as *mut plic::Plic;
        let ctx = PLICCtxWrapper::new(ctx_id);

        let irq = unsafe { (*plic).claim(ctx) };
        irq.map(|irq| irq.get() as usize)
    }

    pub fn complete_irq(&self, irq: usize, ctx_id: usize) {
        let plic = (self.mmio_base + Constant::KERNEL_ADDR_SPACE.start) as *mut plic::Plic;
        let src = PLICSrcWrapper::new(irq);
        let ctx = PLICCtxWrapper::new(ctx_id);
        unsafe { (*plic).complete(ctx, src) };
    }
}

/// Guaranteed to have a PLIC
pub fn scan_plic_device(root: &Fdt) -> Option<PLIC> {
    if let Some(plic_node) = root.find_compatible(&["riscv,plic0", "sifive,plic-1.0.0"]) {
        let plic_reg = plic_node.reg().unwrap().next().unwrap();
        let mmio_base = plic_reg.starting_address as usize;
        let mmio_size = plic_reg.size.unwrap();
        log::info!("plic base_address:{mmio_base:#x}, size:{mmio_size:#x}");
        Some(PLIC::new(mmio_base, mmio_size))
    } else {
        log::error!("[PLIC probe] faild to find plic");
        None
    }
}

#[derive(Debug, Clone, Copy)]
struct PLICSrcWrapper {
    irq: usize,
}

impl PLICSrcWrapper {
    fn new(irq: usize) -> Self {
        Self { irq }
    }
}

impl plic::InterruptSource for PLICSrcWrapper {
    fn id(self) -> core::num::NonZeroU32 {
        core::num::NonZeroU32::try_from(self.irq as u32).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
struct PLICCtxWrapper {
    ctx: usize,
}

impl PLICCtxWrapper {
    fn new(ctx: usize) -> Self {
        Self { ctx }
    }
}

impl plic::HartContext for PLICCtxWrapper {
    fn index(self) -> usize {
        self.ctx
    }
}
