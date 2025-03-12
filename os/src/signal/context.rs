//! context for signal handle
//! see linux's ucontext.h

use super::SigSet;

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
    /// the current context block list
    pub uc_sigmask: SigSet,
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

#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// machine state
pub struct MContext {
    pub user_x: [usize; 32],
    pub fpstate: [usize; 66],
}
