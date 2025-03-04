//! Implementation of [`TrapContext`]
use riscv::register::sstatus::{self, Sstatus,FS, SPP};
use log::info;


///trap context structure containing sstatus, sepc and registers
/// Trap context structure containing sstatus, sepc and registers
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapContext {
    /// user-to-kernel should save:
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus      
    pub sstatus: Sstatus, // 32
    // pub sstatus: usize, // 32
    /// CSR sepc
    pub sepc: usize, // 33

    /// Unlike rCore-tutorial, we don't need to save
    /// trap_handler here, since we will trap back to kernel
    /// and go to trap handler by reloading kernel's ra(through __trap_from_user).
    // pub trap_handler: usize,

    /// kernel-to-user should save:
    ///
    pub kernel_sp: usize, // 34
    ///
    pub kernel_ra: usize, // 35
    ///
    pub kernel_s: [usize; 12], // 36 - 47
    ///
    pub kernel_fp: usize, // 48
}
impl TrapContext {
    ///set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    ///init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        sstatus.set_spp(SPP::User);
        sstatus.set_sie(false);
        sstatus.set_spie(false);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            // saved in ___restore
            kernel_sp: 0,
            kernel_ra: 0,
            kernel_s: [0; 12],
            kernel_fp: 0,
        };
        cx.set_sp(sp);
        cx
    }
    /// set entry point
    pub fn set_entry_point(&mut self, entry: usize) {
        self.sepc = entry;
    }
}
