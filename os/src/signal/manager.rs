//! signal manager
//! every process & thread have a signal manager
//! it is responsible for receving signal and check and handle them

use core::arch::global_asm;

use alloc::collections::vec_deque::VecDeque;
use hal::{addr::VirtAddr, vm::{KernVmSpaceHal, UserVmSpaceHal}};
use log::*;
use crate::{mm::copy_out, signal::{MContext, SigStack, UContext}, processor::processor::{current_task,current_trap_cx}};

use super::{action::KSigAction, SigSet, SIGKILL, SIGSTOP, SIG_NUM};

pub struct SigManager {
    /// Pending signals
    pub pending_sigs: VecDeque<usize>,
    /// bitmap to avoid dup signal
    pub bitmap: SigSet,
    /// Blocked signals
    pub blocked_sigs: SigSet,
    /// Signal handler for every signal
    pub sig_handler: [KSigAction; SIG_NUM + 1],
}

impl SigManager {
    /// create a new signal manager
    pub fn new() -> Self {
        Self {
            pending_sigs: VecDeque::new(),
            bitmap: SigSet::empty(),
            blocked_sigs: SigSet::empty(),
            sig_handler: core::array::from_fn(|signo| KSigAction::new(signo, false)),
        }
    }
    pub fn from_another(sig_manager: &SigManager) -> Self {
        // clean up the pending sigs and blocked sigs
        // use the same action fomr another
        Self {
            pending_sigs: VecDeque::new(),
            bitmap: SigSet::empty(),
            blocked_sigs: SigSet::empty(),
            sig_handler: sig_manager.sig_handler,
        }
    }
    /// signal manager receive a new signal
    pub fn receive(&mut self, signo: usize) {
        if !self.bitmap.contain_sig(signo) {
            self.bitmap.add_sig(signo);
            self.pending_sigs.push_back(signo);
        }
    }
    /// signal manager set signal action
    pub fn set_sigaction(&mut self, signo: usize, sigaction: KSigAction) {
        if signo < SIG_NUM {
            self.sig_handler[signo] = sigaction;
        }
    }
}