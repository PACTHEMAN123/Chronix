//! signal related syscall

use hal::{
    trap::TrapContextHal,
    signal::*,
};
use log::*;

use crate::processor;
use crate::processor::context::SumGuard;
use crate::processor::processor::current_processor;
use crate::signal::*;
use crate::task::current_task;
use crate::processor::processor::current_trap_cx;

/// syscall: kill
pub fn sys_kill(pid: isize, signo: i32) -> isize {
    match pid {
        0 => {
            // sent to every process in the process group of current process
            let task = current_task().unwrap().clone();
            let thread_group = task.thread_group.lock();
            for member in thread_group
                .iter()
                .filter(|t|t.gettid() != task.gettid()) {
                    // skip the current thread
                    member.sig_manager.lock().receive(signo as usize);
                }
        }
        -1 => {
            // sent to every process which current process has permission ( except init proc )
            panic!("[sys_kill] unsupport for sending signal to all process");
        }
        _ if pid < -1 => {
            // sent to every process in process group whose ID is -pid
            panic!("[sys_kill] unsupport for sending signal to specific process group");
        }
        _ if pid > 0 => {
            // sent to the process specified with pid
            let task = current_task().unwrap().clone();
            //assert!(task.gettid() != pid as usize); // should not send to itself
            let thread_group = task.thread_group.lock();
            for member in thread_group
                .iter()
                .filter(|t|t.gettid() == pid as usize) {
                    // skip the current thread
                    info!("{} send signo to {}", task.gettid(), pid);
                    member.sig_manager.lock().receive(signo as usize);
                }
        }
        _ => {}
    }
    0
}


/// syscall: rt_sigaction
pub fn sys_rt_sigaction(signo: i32, action: *const SigAction, old_action: *mut SigAction) -> isize {
    info!(
        "[sys_rt_sigaction]: sig {}, new act ptr {:#x}, old act ptr {:#x}, act size {}",
        signo,
        action as usize,
        old_action as usize,
        core::mem::size_of::<SigAction>()
    );
    if signo < 0 || signo as usize > SIG_NUM {
        info!("[sys_rt_sigaction]: error");
        return -1;
    }

    let task = current_task().unwrap().clone();
    let sig_manager = task.sig_manager.lock();
    let _sum_guard = SumGuard::new();
    info!("[sys_rt_sigaction]: writing old action");
    if old_action as *const u8 != core::ptr::null::<u8>() {
        let k_sig_hand = &sig_manager.sig_handler[signo as usize];
        unsafe {
            if k_sig_hand.is_user {
                old_action.copy_from(&k_sig_hand.sa, 1);
            } else {
                let mut sig_hand = k_sig_hand.sa;
                sig_hand.sa_handler = SIG_DFL;
                old_action.copy_from(&sig_hand as *const SigAction, 1);
            }
        }
    }
    drop(sig_manager);

    info!("[sys_rt_sigaction]: reading new action");
    if action as *const u8 != core::ptr::null::<u8>() {
        let mut sig_action = unsafe { *action };
        let new_sigaction = match sig_action.sa_handler as usize {
            SIG_DFL => KSigAction::new(signo as usize, false),
            SIG_IGN => {
                sig_action.sa_handler = ign_sig_handler as *const () as usize;
                KSigAction {
                    sa: sig_action,
                    is_user: false,
                }
            }
            SIG_ERR => {
                todo!()
            }
            _ => KSigAction {
                sa: sig_action,
                is_user: true,
            },
        };
        log::info!(
                "[sys_rt_sigaction]: sig {}, set new sig handler {:#x}, sa_mask {:?}, sa_flags: {:#x}, sa_restorer: {:#x}",
                signo,
                new_sigaction.sa.sa_handler as *const usize as usize,
                new_sigaction.sa.sa_mask[0],
                new_sigaction.sa.sa_flags,
                new_sigaction.sa.sa_restorer,
            );
        task.set_sigaction(signo as usize, new_sigaction);
    }
    0
}

const SIGBLOCK: i32 = 0;
const SIGUNBLOCK: i32 = 1;
const SIGSETMASK: i32 = 2;

/// syscall: rt_sigprocmask
pub fn sys_rt_sigprocmask(how: i32, set: *const u32, old_set: *mut SigSet) -> isize {
    info!("[sys_rt_sigprocmask]: how: {}", how);
    let task = current_task().unwrap().clone();
    let mut sig_manager = task.sig_manager.lock();
    if old_set as usize != 0 {
        let _sum_guard = SumGuard::new();
        unsafe {
            *old_set = sig_manager.blocked_sigs;
            debug!("[sys_rt_sigprocmask] old set: {:?}", sig_manager.blocked_sigs);
        }
    }
    if set as usize == 0 {
        debug!("arg set is null");
        return 0;
    }
    let _sum_guard = SumGuard::new();
    
    let new_sig_mask = unsafe { SigSet::from_bits(*set as usize).unwrap() };
    log::info!(
        "[sys_rt_sigprocmask] how {}, new sig mask: {:?}",
        how,
        new_sig_mask
    );
    match how {
        SIGBLOCK => {
            sig_manager.blocked_sigs |= new_sig_mask;
        }
        SIGUNBLOCK => {
            sig_manager.blocked_sigs.remove(new_sig_mask);
        }
        SIGSETMASK => {
            sig_manager.blocked_sigs = new_sig_mask;
        }
        _ => {
            return -1;
        }
    };
    0
}

/// syscall: rt_sigreturn
pub fn sys_rt_sigreturn() -> isize {
    info!("[sys_rt_sigreturn]: into");
    // read from user context
    let _sum_guard = SumGuard::new();
    let task = current_task().unwrap();
    let ucontext_ptr = task.sig_ucontext_ptr();
    let ucontext = unsafe {
        *(ucontext_ptr as *const UContext)
    };
    let mut sig_manager = task.sig_manager.lock();
    // restore the old sig mask
    sig_manager.blocked_sigs = SigSet::from_bits_truncate(ucontext.uc_sigmask);
    // restore the old context (todo: restore signal stack)
    let cx = current_trap_cx(current_processor());
    ucontext.restore_old_context(cx);
    0
}