//! signal manager
//! every process & thread have a signal manager
//! it is responsible for receving signal and check and handle them

use core::arch::global_asm;

use alloc::collections::vec_deque::VecDeque;
use hal::{addr::VirtAddr, signal::*};
use crate::mm::vm::{KernVmSpaceHal, UserVmSpaceHal};
use log::*;
use crate::{mm::copy_out, processor::processor::{current_task,current_trap_cx}};

use super::{action::KSigAction, SigInfo, SigSet, SIGKILL, SIGSTOP, SIG_NUM};

pub struct SigManager {
    /// Pending signals
    pub pending_sigs: VecDeque<SigInfo>,
    /// bitmap to avoid dup signal
    pub bitmap: SigSet,
    /// Blocked signals
    pub blocked_sigs: SigSet,
    /// Signal handler for every signal
    pub sig_handler: [KSigAction; SIG_NUM + 1],
    /// Wake up signals
    pub wake_sigs: SigSet,
}

impl SigManager {
    /// create a new signal manager
    pub fn new() -> Self {
        Self {
            pending_sigs: VecDeque::new(),
            bitmap: SigSet::empty(),
            blocked_sigs: SigSet::empty(),
            sig_handler: core::array::from_fn(|signo| KSigAction::new(signo, false)),
            wake_sigs: SigSet::empty(),
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
            wake_sigs: SigSet::empty(),
        }
    }
    /// signal manager receive a new signal
    pub fn receive(&mut self, signo_info: SigInfo) {
        if !self.bitmap.contain_sig(signo_info.si_signo) {
            self.bitmap.add_sig(signo_info.si_signo);
            self.pending_sigs.push_back(signo_info);
        }
    }
    /// check if there is any expected SigInfo in the pending_sigs
    pub fn check_pending(&mut self, expected: SigSet) -> Option<SigInfo> {
        let x = self.bitmap & expected;
        if x.is_empty() {
            return None;
        }
        for i in 0..self.pending_sigs.len() {
            if x.contain_sig(self.pending_sigs[i].si_signo) {
                return Some(self.pending_sigs[i]);
            }
        }
        // log::warn!("[SigManager] check_pending failed, should not happen");
        None
    }
    /// bool flag to check if there is any pending signal expected
    pub fn check_pending_flag(&self, expected: SigSet) -> bool {
        !(expected & self.bitmap).is_empty()
    }
    /// signal manager set signal action
    pub fn set_sigaction(&mut self, signo: usize, sigaction: KSigAction) {
        if signo < SIG_NUM {
            self.sig_handler[signo] = sigaction;
        }
    }
    /// dequeue a specific signal 
    pub fn dequeue_expected(&mut self, expected: SigSet) -> Option<SigInfo> {
        let sig_set = self.bitmap & expected;
        if sig_set.is_empty() {
            return None;
        }
        for i in 0..self.pending_sigs.len() {
            let sig = self.pending_sigs[i].si_signo;
            if sig_set.contain_sig(sig) {
                self.bitmap.remove_sig(sig);
                return self.pending_sigs.remove(i);
            }
        }
        log::warn!("[SigManager] dequeue_expected failed, should not happen");
        None
    }
    
}