use core::sync::atomic::{AtomicBool, Ordering};

use crate::{constant::{Constant, ConstantsHal}, entry::BOOT_STACK, println, timer::{Timer, TimerHal}};

const VIRT_RAM_OFFSET: usize = Constant::KERNEL_ADDR_SPACE.start;


#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        r"
        csrrd        $a0, 0x20                    # cpuid
        addi.d       $t0, $a0, 1                  # t0 = hart_id + 1
        la.global    $sp, {boot_stack}
        li.d         $t1, {boot_stack_size}
        mul.d        $t0, $t1, $t0                # t0 = (hart_id + 1) * boot_stack_size
        add.d        $sp, $sp, $t0

        ori          $t0, $zero, 0x1     # CSR_DMW1_PLV0
        lu52i.d      $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
        csrwr        $t0, 0x180          # LOONGARCH_CSR_DMWIN0
        ori          $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
        lu52i.d      $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
        csrwr        $t0, 0x181          # LOONGARCH_CSR_DMWIN1

        # Enable PG 
        li.w		 $t0, 0xb0		              # PLV=0, IE=0, PG=1
        csrwr	  	 $t0, 0x0                     # LOONGARCH_CSR_CRMD
        li.w	 	 $t0, 0x00		              # PLV=0, PIE=0, PWE=0
        csrwr		 $t0, 0x1                     # LOONGARCH_CSR_PRMD
        li.w		 $t0, 0x00		              # FPE=0, SXE=0, ASXE=0, BTE=0
        csrwr		 $t0, 0x2                     # LOONGARCH_CSR_EUEN

        li.d         $t2, {virt_ram_offset}       
        or           $sp, $sp, $t2
        la.global    $a2, {entry}
        or           $a2, $a2, $t2
        jirl         $zero, $a2, 0                # call rust_main
        ",
        boot_stack_size = const Constant::KERNEL_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        virt_ram_offset = const VIRT_RAM_OFFSET,
        entry = sym rust_main
    );
}

pub static FIRST_PROCESSOR: AtomicBool = AtomicBool::new(true);

pub(crate) fn rust_main(id: usize) {
    tlb_init();
    if FIRST_PROCESSOR.compare_exchange(true, false, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
        super::clear_bss();
        crate::console::init();
        print_info();
    }
    unsafe { super::_main_for_arch(id); }
}

fn print_info() {
    println!(r" ____  _                 _      _   _    _    _     
/ ___|(_)_ __ ___  _ __ | | ___| | | |  / \  | |    
\___ \| | '_ ` _ \| '_ \| |/ _ \ |_| | / _ \ | |    
 ___) | | | | | | | |_) | |  __/  _  |/ ___ \| |___ 
|____/|_|_| |_| |_| .__/|_|\___|_| |_/_/   \_\_____|
                  |_|                               ");
    println!("PA_LEN: {}", loongArch64::cpu::get_palen());
    println!("VA_LEN: {}", loongArch64::cpu::get_valen());
    println!("Frequency: {} Hz", Timer::get_timer_freq());
    println!("");
}

core::arch::global_asm!(
    r"
    .equ LA_CSR_PGDL,          0x19    /* Page table base address when VA[47] = 0 */
    .equ LA_CSR_PGDH,          0x1a    /* Page table base address when VA[47] = 1 */
    .equ LA_CSR_PGD,           0x1b    /* Page table base */
    .equ LA_CSR_TLBRENTRY,     0x88    /* TLB refill exception entry */
    .equ LA_CSR_TLBRBADV,      0x89    /* TLB refill badvaddr */
    .equ LA_CSR_TLBRERA,       0x8a    /* TLB refill ERA */
    .equ LA_CSR_TLBRSAVE,      0x8b    /* KScratch for TLB refill exception */
    .equ LA_CSR_TLBRELO0,      0x8c    /* TLB refill entrylo0 */
    .equ LA_CSR_TLBRELO1,      0x8d    /* TLB refill entrylo1 */
    .equ LA_CSR_TLBREHI,       0x8e    /* TLB refill entryhi */
    .globl _tlb_fill
    .align 12
    _tlb_fill:
        csrwr   $t0, LA_CSR_TLBRSAVE
        csrrd   $t0, LA_CSR_PGD
        lddir   $t0, $t0, 3
        lddir   $t0, $t0, 2
        lddir   $t0, $t0, 1
        ldpte   $t0, 0
        ldpte   $t0, 1
        tlbfill
        csrrd   $t0, LA_CSR_TLBRSAVE
        ertn
    "
);


/// Sv48 mode
fn tlb_init() {

    use loongArch64::register::*;

    tlbidx::set_ps(Constant::PAGE_SIZE_BITS);
    stlbps::set_ps(Constant::PAGE_SIZE_BITS);
    tlbrehi::set_ps(Constant::PAGE_SIZE_BITS);
    asid::set_asid_width(0);
    pwcl::set_pte_width(8);
    pwcl::set_ptbase(12);
    pwcl::set_ptwidth(9);
    pwcl::set_dir1_base(21);
    pwcl::set_dir1_width(9);
    pwcl::set_dir2_base(30);
    pwcl::set_dir2_width(9);
    pwch::set_dir3_base(39);
    pwch::set_dir3_width(9);
    pwch::set_dir4_base(0);
    pwch::set_dir4_width(0);
    unsafe extern "C" {
        fn _tlb_fill();
    }
    assert!((_tlb_fill as usize) % 4096 == 0);
    
    tlbrentry::set_tlbrentry(_tlb_fill as usize & ((1 << Constant::PA_WIDTH) - 1));

    unsafe {
        core::arch::asm!("invtlb 0x0, $r0, $r0"); //clear tlb
    }
}
