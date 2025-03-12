//! signal action

use super::*;


#[derive(Clone, Copy)]
#[repr(C)]
/// signal action struct under riscv-linux arch
pub struct SigAction {
    pub sa_handler: usize,
    pub sa_flags: u32,
    pub sa_restorer: usize,
    pub sa_mask: [SigSet; 1],
}

#[derive(Clone, Copy)]
/// signal action warpper for kernel
pub struct KSigAction {
    /// inner sigaction
    pub sa: SigAction,
    /// is user defined?
    pub is_user: bool,
}

impl KSigAction {
    pub fn new(signo: usize, is_user_defined: bool) -> Self {
        let sa_handler = match signo {
            SIGHUP => term_sig_handler,
            SIGINT => term_sig_handler,
            SIGILL => term_sig_handler,
            SIGABRT => term_sig_handler,
            SIGBUS => term_sig_handler,
            SIGKILL => term_sig_handler,
            SIGSEGV => term_sig_handler,
            SIGALRM => term_sig_handler,
            SIGTERM => term_sig_handler,
            SIGCHLD => ign_sig_handler,
            SIGSTOP => stop_sig_handler,
            _ => ign_sig_handler,
        } as *const () as usize;
        Self {
            is_user: is_user_defined,
            sa: SigAction {
                sa_handler,
                sa_mask: [SigSet::from_bits(0).unwrap(); 1],
                sa_flags: 0,
                sa_restorer: 0,
            }
        }
    }
}