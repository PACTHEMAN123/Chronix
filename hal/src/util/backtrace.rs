use crate::{println, trap::{TrapContext, TrapContextHal}};

pub fn backtrace() {
    unsafe extern "C" {
        fn stext();
        fn etext();
    }
    println!("backtrace:");
    unsafe {
        
        let mut current_pc: usize;
        let mut current_fp: usize;

        #[cfg(target_arch="riscv64")]
        core::arch::asm!(
            r"
            mv {}, ra
            mv {}, fp
            ",
            out(reg) current_pc,
            out(reg) current_fp
        );

        #[cfg(target_arch="loongarch64")]
        core::arch::asm!(
            r"
            move {}, $ra
            move {}, $fp
            ",
            out(reg) current_pc,
            out(reg) current_fp
        );

        while current_pc >= stext as usize && current_pc <= etext as usize && current_fp != 0 {
            println!("{:#x}", current_pc - size_of::<usize>());
            current_fp = *(current_fp as *const usize).offset(-2);
            current_pc = *(current_fp as *const usize).offset(-1);
        }
    }
}