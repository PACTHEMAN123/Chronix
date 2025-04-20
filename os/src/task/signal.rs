//! task signal related implement

use core::{future::Future, pin::Pin, task::{Context, Poll}};

use alloc::sync::Arc;
use fatfs::info;
use hal::{addr::VirtAddr, println, signal::{sigreturn_trampoline_addr, UContext, UContextHal}, trap::TrapContextHal};

use crate::{mm::{copy_out, vm::UserVmSpaceHal}, signal::{KSigAction, SigInfo, SigSet, SIGKILL, SIGSTOP}};

use super::task::TaskControlBlock;


/// for the signal mechanism
impl TaskControlBlock {
    /// once the leader thread change the sig action
    /// all its follower should change
    pub fn set_sigaction(&self, signo: usize, sigaction: KSigAction) {
        //info!("[TCB] sync all child thread sigaction");
        self.sig_manager.lock().set_sigaction(signo, sigaction);
        self.with_mut_children(|children| {
            if children.len() == 0 {
                return;
            }
            for child in children.values() {
                child.sig_manager.lock().set_sigaction(signo, sigaction);
            }
        })
    }
    /// set self's wake up signals
    /// when these signals arrive it should wake itself up
    pub fn set_wake_up_sigs(&self, sigs: SigSet) {
        assert!(self.is_interruptable());
        self.with_mut_sig_manager(|manager| {
            manager.wake_sigs = sigs | SigSet::SIGKILL | SigSet::SIGSTOP
        })
    }
    /// receive function at TCB level
    /// as we may need to wake up a task when wake up signal come
    pub fn recv_sigs(&self, signo: SigInfo) {
        //info!("[TCB]: tid {} recv signo {}", self.gettid(), signo);
        self.with_mut_sig_manager(|manager| {
            manager.receive(signo);
            if manager.wake_sigs.contain_sig(signo.si_signo) && self.is_interruptable() {
                //info!("[TCB]: tid {} has been wake up", self.gettid());
                self.wake();
            }
        });
    }
    /// Unix has two types of signal: Process level and Thread level
    /// in Process-level, all threads in the same process share the same signal mask
    pub fn recv_sigs_process_level(&self, sig_info: SigInfo) {
        log::info!("[TCB::recv_sigs_process_level]: tid {} recv signo {} at process level",self.tid(),sig_info.si_signo);
        self.with_mut_thread_group(|tg| {
            let mut signal_delivered = false;
            for thread in tg.iter() {
                if thread.sig_manager.lock().blocked_sigs.contain_sig(sig_info.si_signo) {
                    continue;
                }
                thread.recv_sigs(sig_info);
                signal_delivered = true;
                break;
            } 
            if !signal_delivered {
                let task = tg.iter().next().unwrap();
                task.recv_sigs(sig_info);
            }
        })
    }
    /// signal manager should check the signal queue
    /// before a task return form kernel to user
    /// and make correspond handle action
    pub fn check_and_handle(&self) {
        self.with_mut_sig_manager(|sig_manager| {
            // check the signal, try to find first handle signal
            if sig_manager.pending_sigs.is_empty() {
                return;
            }
            let len = sig_manager.pending_sigs.len();
            let mut cnt = 0;
            let mut signo = SigInfo{
                si_signo: 0,
                si_code: 0,
                si_pid: None
            };
            while cnt < len {
                signo = sig_manager.pending_sigs.pop_front().unwrap();
                cnt += 1;
                // block the signals
                if signo.si_signo != SIGKILL && signo.si_signo != SIGSTOP && sig_manager.blocked_sigs.contain_sig(signo.si_signo) {
                    //info!("[SIGHANDLER] signal {} blocked", signo);
                    sig_manager.pending_sigs.push_back(signo);
                    continue;
                }
                //info!("[SIGHANDLER] receive signal {}", signo);
                break;
            }
            // handle a signal
            assert!(signo.si_signo != 0);
            let sig_action = sig_manager.sig_handler[signo.si_signo];
            let trap_cx = self.get_trap_cx();
            if sig_action.is_user {
                let old_blocked_sigs = sig_manager.blocked_sigs; // save for later restore
                sig_manager.blocked_sigs.add_sig(signo.si_signo);
                sig_manager.blocked_sigs |= sig_action.sa.sa_mask[0];
                // save fx state
                trap_cx.fx_encounter_signal();
                // push the current Ucontext into user stack
                // (todo) notice that user may provide signal stack
                // but now we dont support this flag
                let sp = *trap_cx.sp();
                let new_sp = sp - size_of::<UContext>();
                let ucontext = UContext::save_current_context(old_blocked_sigs.bits(), trap_cx);
                let ucontext_bytes: &[u8] = unsafe {
                    core::slice::from_raw_parts(
                        &ucontext as *const UContext as *const u8,
                        core::mem::size_of::<UContext>(),
                    )
                };
                // println!("copy_out to {:#x}", new_sp);
                copy_out(&mut self.vm_space.lock(), VirtAddr(new_sp), ucontext_bytes);
                self.set_sig_ucontext_ptr(new_sp);

                // set the current trap cx sepc to reach user handler
                // log::info!("set signal handler sepc: {:x}", sig_action.sa.sa_handler as *const usize as usize);
                *trap_cx.sepc() = sig_action.sa.sa_handler as *const usize as usize;
                // a0
                trap_cx.set_arg_nth(0, signo.si_signo);
                // sp used by sys_sigreturn to restore ucontext
                *trap_cx.sp() = new_sp;
                // ra: when user signal handler ended, return to sigreturn_trampoline
                // which calls sys_sigreturn
                *trap_cx.ra() = sigreturn_trampoline_addr();
            } else {
                let handler = unsafe {
                    core::mem::transmute::<*const (), fn(usize)>(
                        sig_action.sa.sa_handler as *const (),
                    )
                };
                handler(signo.si_signo);
            }
        });
    }
}

/// the future that check if recv expect signal
pub struct IntrBySignalFuture {
    /// the task needed to check
    pub task: Arc<TaskControlBlock>,
    /// current signal mask
    pub mask: SigSet,
}

impl Future for IntrBySignalFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let has_signal = !(self.task.sig_manager.lock().bitmap & !self.mask).is_empty();
        if has_signal {
            log::warn!("[IntrBySignalFuture] received interupt signal");
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}