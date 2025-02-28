use crate::mm::VirtAddr;

pub unsafe fn sfence_vma_vaddr(va: VirtAddr) {
    core::arch::asm!("sfence.vma {}, x0", in(reg) usize::from(va), options(nostack));
}
pub unsafe fn sfence_vma_all() {
    core::arch::asm!("sfence.vma");
}

pub mod entry;