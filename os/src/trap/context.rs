//! Implementation of [`TrapContext`]
use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
#[derive(Debug)]
///trap context structure containing sstatus, sepc and registers
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus      32
    pub sstatus: Sstatus,
    /// CSR sepc
    pub sepc: usize,     //33
    /// kernel stack
    pub kernel_sp: usize,   //34
    /// now move task_context to here
    /// 35
    pub kernel_ra: usize,
    /// 36-47
    pub kernel_s: [usize; 12],       
    /// 48
    pub kernel_fp: usize,            
    // leave 49 for tp
    // leave float regs
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
        kernel_sp: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_sp,
            kernel_ra: 0,
            kernel_s: [0; 12],
            kernel_fp: 0,
        };
        cx.set_sp(sp);
        cx
    }
}
