//! signal manager
//! every process & thread have a signal manager
//! it is responsible for receving signal and check and handle them

use core::arch::global_asm;

use alloc::collections::vec_deque::VecDeque;
use log::*;
use crate::{logging, mm::{copy_out, VirtAddr}, signal::{MContext, SigStack, UContext}, task::{current_task, processor::current_trap_cx}};

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

global_asm!(include_str!("trampoline.S"));

unsafe extern "C" {
    unsafe fn sigreturn_trampoline();
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
    /// signal manager should check the signal queue
    /// before a task return form kernel to user
    /// and make correspond handle action
    pub fn check_and_handle(&mut self) {
        // check the signal, try to find first handle signal
        if self.pending_sigs.is_empty() {
            return;
        }
        let len = self.pending_sigs.len();
        let mut cnt = 0;
        let mut signo: usize = 0;
        while cnt < len {
            signo = self.pending_sigs.pop_front().unwrap();
            cnt += 1;
            // block the signals
            if signo != SIGKILL && signo != SIGSTOP && self.blocked_sigs.contain_sig(signo) {
                info!("[SIGHANDLER] signal {} blocked", signo);
                self.pending_sigs.push_back(signo);
                continue;
            }
            info!("[SIGHANDLER] receive signal {}", signo);
            break;
        }
        // handle a signal
        assert!(signo != 0);
        let sig_action = self.sig_handler[signo];
        if sig_action.is_user {
            let old_blocked_sigs = self.blocked_sigs; // save for later restore
            self.blocked_sigs.add_sig(signo);
            self.blocked_sigs |= sig_action.sa.sa_mask[0];

            // push the current Ucontext into user stack
            // (todo) notice that user may provide signal stack
            // but now we dont support this flag
            let sp = current_trap_cx().x[2];
            let new_sp = sp - size_of::<UContext>();
            let mut ucontext = UContext {
                uc_flags: 0,
                uc_link: 0,
                uc_stack: SigStack::new(),
                uc_sigmask: old_blocked_sigs,
                uc_sig: [0; 16],
                uc_mcontext: MContext {
                    user_x: current_trap_cx().x,
                    fpstate: [0; 66],
                },
            };
            ucontext.uc_mcontext.user_x[0] = current_trap_cx().sepc;
            let ucontext_bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    &ucontext as *const UContext as *const u8,
                    core::mem::size_of::<UContext>(),
                )
            };
            copy_out(&current_task().unwrap().vm_space.lock().page_table, VirtAddr(new_sp), ucontext_bytes);
            current_task().unwrap().set_sig_ucontext_ptr(new_sp);

            // set the current trap cx sepc to reach user handler
            current_trap_cx().sepc = sig_action.sa.sa_handler as *const usize as usize;
            // a0
            current_trap_cx().x[10] = signo;
            // sp used by sys_sigreturn to restore ucontext
            current_trap_cx().x[2] = new_sp;
            // ra: when user signal handler ended, return to sigreturn_trampoline
            // which calls sys_sigreturn
            current_trap_cx().x[1] = sigreturn_trampoline as usize;
            // other important regs
            current_trap_cx().x[4] = ucontext.uc_mcontext.user_x[4];
            current_trap_cx().x[3] = ucontext.uc_mcontext.user_x[3];
        } else {
            let handler = unsafe {
                core::mem::transmute::<*const (), fn(usize)>(
                    sig_action.sa.sa_handler as *const (),
                )
            };
            handler(signo);
        }
    }

}