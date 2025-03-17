use core::sync::atomic::{AtomicBool, Ordering};
use crate::{constant::{Constant, ConstantsHal}, entry::BOOT_STACK};

#[repr(C, align(4096))]
pub struct BootPageTable([u64; Constant::PTES_PER_PAGE]);

pub static mut BOOT_PAGE_TABLE: BootPageTable = {
    let mut arr: [u64; Constant::PTES_PER_PAGE] = [0; Constant::PTES_PER_PAGE];
    arr[2] = (0x80000 << 10) | 0xcf;
    arr[256] = (0x00000 << 10) | 0xcf;
    arr[258] = (0x80000 << 10) | 0xcf;
    BootPageTable(arr)
};

const VIRT_RAM_OFFSET: usize = Constant::KERNEL_ADDR_SPACE.start;

#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start(id: usize) -> ! {
    core::arch::naked_asm!(
        // 1. set boot stack
        // a0 = processor_id
        // sp = boot_stack + (hartid + 1) * 64KB
        "
            addi    t0, a0, 1
            li      t1, {boot_stack_size}
            mul     t0, t0, t1                // t0 = (hart_id + 1) * boot_stack_size
            la      sp, {boot_stack}
            add     sp, sp, t0                // set boot stack
        ",
        // 2. enable sv39 page table
        // satp = (8 << 60) | PPN(page_table)
        "
            la      t0, {page_table}
            srli    t0, t0, 12
            li      t1, 8 << 60
            or      t0, t0, t1
            csrw    satp, t0
            sfence.vma
        ",
        // 3. jump to rust_main
        // add virtual address offset to sp and pc
        "
            li      t2, {virt_ram_offset}
            or      sp, sp, t2
            la      a2, {entry}
            or      a2, a2, t2
            jalr    a2                      // call rust_main
        ",
        boot_stack_size = const Constant::KERNEL_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        page_table = sym BOOT_PAGE_TABLE,
        entry = sym rust_main,
        virt_ram_offset = const VIRT_RAM_OFFSET,
    )
}

pub static FIRST_PROCESSOR: AtomicBool = AtomicBool::new(true);

pub(crate) fn rust_main(id: usize) {
    if FIRST_PROCESSOR.load(Ordering::Acquire) {
        FIRST_PROCESSOR.store(false, Ordering::Release);
        super::clear_bss();
        crate::console::init();
    }
    unsafe { super::_main_for_arch(id) };
}