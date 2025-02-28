pub unsafe fn sfence_vma_vaddr(va: usize) {
    core::arch::asm!("sfence.vma {}, x0", in(reg) va, options(nostack));
}
pub unsafe fn sfence_vma_all() {
    core::arch::asm!("sfence.vma");
}

pub mod entry;