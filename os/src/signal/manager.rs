//! signal manager
//! every process & thread have a signal manager
//! it is responsible for receving signal and check and handle them

use core::arch::global_asm;

use alloc::collections::{btree_map::BTreeMap, vec_deque::VecDeque};
use hal::{addr::VirtAddr, signal::*};
use crate::mm::vm::{KernVmSpaceHal, UserVmSpaceHal};
use log::*;
use crate::{mm::copy_out, processor::processor::{current_task,current_trap_cx}};

use super::{action::KSigAction, get_default_handler, ign_sig_handler, SigInfo, SigSet, SIGKILL, SIGRTMAX, SIGRTMIN, SIGSTOP};

pub struct SigManager {
    /// Pending standard signals
    pub pending_sigs: VecDeque<SigInfo>,
    /// Pending real-time signals
    /// low-numbered signals have highest priority.
    /// Multiple instances of real-time signals can be queued
    pub pending_rt_sigs: BTreeMap<usize, VecDeque<SigInfo>>,
    /// bitmap to avoid dup standard signal
    pub bitmap: SigSet,
    /// Blocked signals
    pub blocked_sigs: SigSet,
    /// Signal handler for every signal
    pub sig_handler: [KSigAction; SIGRTMAX + 1],
    /// Wake up signals
    pub wake_sigs: SigSet,
}

impl SigManager {
    /// create a new signal manager
    pub fn new() -> Self {
        Self {
            pending_sigs: VecDeque::new(),
            pending_rt_sigs: BTreeMap::new(),
            bitmap: SigSet::empty(),
            blocked_sigs: SigSet::empty(),
            sig_handler: core::array::from_fn(|signo| KSigAction::new(signo, false)),
            wake_sigs: SigSet::empty(),
        }
    }
    pub fn from_another(sig_manager: &SigManager) -> Self {
        // clean up the pending sigs and blocked sigs
        // use the same action from another
        Self {
            pending_sigs: VecDeque::new(),
            pending_rt_sigs: BTreeMap::new(),
            bitmap: SigSet::empty(),
            blocked_sigs: SigSet::empty(),
            sig_handler: sig_manager.sig_handler,
            wake_sigs: SigSet::empty(),
        }
    }
    /// signal manager receive a new signal
    /// according to linux manual, a process will only receive
    /// the information associated with the first instance of the signal.
    pub fn receive(&mut self, signo_info: SigInfo) {
        let signo = signo_info.si_signo;
        if signo < SIGRTMIN {
            if !self.bitmap.contain_sig(signo) {
                self.bitmap.add_sig(signo);
                self.pending_sigs.push_back(signo_info);
            }
        } else {
            assert!(signo >= SIGRTMIN);
            assert!(signo <= SIGRTMAX);
            self.pending_rt_sigs
                .entry(signo)
                .or_insert_with(VecDeque::new)
                .push_back(signo_info);
        }
        
    }

    /// check if there is any expected SigInfo in the pending_sigs
    /// if found, return the first match
    pub fn check_pending(&mut self, expected: SigSet) -> Option<SigInfo> {
        let x = self.bitmap & expected;
        if x.is_empty() {
            // no expected standard signal found
            // check real-time signals
            for (&signo, queue) in self.pending_rt_sigs.iter() {
                if x.contain_sig(signo) {
                    return queue.front().cloned()
                }
            }
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
    /// if exist, return true
    pub fn check_pending_flag(&self, expected: SigSet) -> bool {
        let x = self.bitmap & expected;
        if x.is_empty() {
            for (&signo, _) in self.pending_rt_sigs.iter() {
                if x.contain_sig(signo) {
                    return true
                }
            }
            return false
        }
        true
    }
    /// signal manager set signal action
    pub fn set_sigaction(&mut self, signo: usize, sigaction: KSigAction) {
        if signo == SIGSTOP || signo == SIGKILL {
            warn!("SIGKILL or SIGSTOP cannot be caught or ignored");
            return;
        }
        if signo <= SIGRTMAX {
            self.sig_handler[signo] = sigaction;
        }
    }
    /// dequeue a pending signal to handle
    /// called by `check_and_handle`
    pub fn dequeue_one(&mut self) -> Option<SigInfo> {
        // If both standard and real-time signals are pending for a process,
        // Chronix, like many other implementations, gives priority to standard signals
        // stage1: check standard signals
        let len = self.pending_sigs.len();
        let mut cnt = 0usize;
        while cnt < len {
            let sig = self.pending_sigs.pop_front().unwrap();
            cnt += 1;
            if sig.si_signo != SIGKILL && sig.si_signo != SIGSTOP && self.blocked_sigs.contain_sig(sig.si_signo) {
                // cannot handle currently, push back to wait for unblock
                self.pending_sigs.push_back(sig);
                continue;
            }
            self.bitmap.remove_sig(sig.si_signo);
            log::info!("[SigManager] dequeue standard signal {:?}", sig);
            return Some(sig);
        }
        assert!(cnt == len);
        // stage2: check real-time signals
        for (&signo, queue) in self.pending_rt_sigs.iter_mut() {
            assert!(signo >= SIGRTMIN);
            assert!(signo <= SIGRTMAX);
            if self.blocked_sigs.contain_sig(signo) {
                continue;
            }
            if let Some(sig) = queue.pop_front() {
                log::info!("[SigManager] dequeue real-time signal {:?}", sig);
                return Some(sig);
            }
        }
        log::debug!("[SigManager] no signals to be handled");
        None
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
    /// reset the signal manager
    /// see https://man7.org/linux/man-pages/man7/signal.7.html
    pub fn reset_on_exec(&mut self) {
        // 1. Signal dispositions
        // During an execve(2), the dispositions of handled
        // signals are reset to the default; the dispositions of ignored
        // signals are left unchanged.
        for signo in 1..SIGRTMAX {
            let old_action = self.sig_handler[signo];
            if old_action.sa.sa_handler == ign_sig_handler as *const() as usize {
                // handler is ignore, 2 cases
                // 1. default handler is IGN
                // 2. default handler is not IGN, but explictly set to SIG_IGN
                // for both case, left unchanged
                continue;
            } else {
                let new_action = KSigAction::new(signo, false);
                self.sig_handler[signo] = new_action;
            }
        }
        // 2. Signal mask and pending signals
        // the signal mask is preserved across execve(2).
        // the pending signal set is preserved across an execve(2).
    }
}