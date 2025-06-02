//! riscv implement for signal HAL

///// signal context

use super::UContextHal;
use crate::{constant::{Constant, ConstantsHal}, trap::TrapContext};

core::arch::global_asm!(include_str!("trampoline.S"));

#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// machine state
pub struct MContext {
    pub user_x: [usize; 32],
    pub fpstate: [usize; 66],
}

impl MContext  {
    pub fn get_tp(&self) -> usize {
        self.user_x[4]
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// UContext: 
pub struct UContext {
    /// (todos) some flags?
    pub uc_flags: usize,
    /// when return from this UContext
    /// use the pointed UContext to restore
    pub uc_link: usize,
    /// the SigStack current context using
    pub uc_stack: SigStack,
    /// the current context block list: SigSet
    pub uc_sigmask: usize,
    /// (todo) align to the call standard
    pub uc_sig: [usize; 16],
    /// save machine state
    pub uc_mcontext: MContext, 
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// signal stack
pub struct SigStack {
    /// base address of stack
    pub ss_sp: usize,
    /// flags
    pub ss_flags: i32,
    /// stack size (num of bytes)
    pub ss_size: usize,
}

impl SigStack {
    pub fn new() -> Self {
        Self {
            ss_sp: 0,
            ss_flags: 0,
            ss_size: 0,
        }
    }
}

impl UContextHal for UContext {
    fn save_current_context(old_blocked_sigs: usize, cx: &TrapContext) -> Self {
        let uc_stack = SigStack::new();
        let mut ucx = Self {
            uc_flags: 0,
            uc_link: 0,
            uc_stack: uc_stack,
            uc_sigmask: old_blocked_sigs,
            uc_sig: [0; 16],
            uc_mcontext: MContext { user_x: cx.x, fpstate: [0; 66]},
        };
        ucx.uc_mcontext.user_x[0] = cx.sepc;
        ucx
    }
    fn restore_old_context(&self, cx: &mut TrapContext) {
        cx.sepc = self.uc_mcontext.user_x[0];
        cx.x = self.uc_mcontext.user_x;
    }
}
 
pub fn sigreturn_trampoline_addr() -> usize {
    Constant::SIGRET_TRAMPOLINE_BOTTOM
}