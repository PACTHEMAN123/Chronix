//! signal related syscall

use hal::instruction::{Instruction, InstructionHal};
use hal::println;
use hal::{
    trap::TrapContextHal,
    signal::*,
};
use log::*;
use super::{SysError,SysResult};
use crate::processor;
use crate::processor::context::SumGuard;
use crate::processor::processor::current_processor;
use crate::signal::*;
use crate::task::{current_task,INITPROC_PID};
use crate::processor::processor::current_trap_cx;
use crate::task::manager::{PROCESS_GROUP_MANAGER, TASK_MANAGER};
use crate::timer::ffi::TimeSpec;
use crate::timer::timed_task::suspend_timeout;
use crate::utils::suspend_now;

/// syscall: kill
pub fn sys_kill(pid: isize, signo: i32) -> SysResult {
    log::debug!("[sys_kill]: sending signo: {} to pid: {}", signo, pid);
    let task = current_task().unwrap().clone();
    let pgid = task.pgid();
    match pid {
        0 => {
            // sent to every process in the process group of current process
            for process in PROCESS_GROUP_MANAGER
            .get_group(pgid)
            .unwrap()
            .into_iter()
            .map(|inner| inner.upgrade().unwrap())
            {
                process.recv_sigs_process_level(
                    SigInfo {
                        si_signo: signo as usize,
                        si_code: SigInfo::USER,
                        si_pid: Some(pgid)
                    }
                );
            }
        }
        -1 => {
            // sent to every process which current process has permission ( except init proc )
            //panic!("[sys_kill] unsupport for sending signal to all process");
            TASK_MANAGER.for_each_task(|task|{
                if task.tid() == INITPROC_PID {
                }
                if signo != 0 && task.is_leader(){
                    task.recv_sigs_process_level(
                        SigInfo { si_signo: signo as usize, si_code: SigInfo::USER, si_pid: Some(task.pid()) },
                    );
                }
            });
        }
        _ if pid < -1 => {
            // sent to every process in process group whose ID is -pid
            //panic!("[sys_kill] unsupport for sending signal to specific process group");
            let inner_pid = -pid as usize;
            for task in PROCESS_GROUP_MANAGER
            .get_group(pgid)
            .unwrap()
            .into_iter()
            .map(|t| t.upgrade().unwrap())
            {
                if task.tid() == inner_pid {
                    task.recv_sigs_process_level(SigInfo { si_signo: signo as usize, si_code: SigInfo::USER, si_pid: Some(task.pgid()) });
                }
            }
        }
        _ if pid > 0 => {
            // sent to the process specified with pid
            //assert!(task.gettid() != pid as usize); // should not send to itself
            if let Some(task) = TASK_MANAGER.get_task(pid as usize) {
                if task.is_leader() {
                    task.recv_sigs_process_level(
                        SigInfo { si_signo: signo as usize, si_code: SigInfo::USER, si_pid: Some(task.pid()) },
                    );
                }else {
                    // todo standard error
                    return Err(SysError::ESRCH);
                }
            }else {
                return Err(SysError::ESRCH);
            }
        }
        _ => {}
    }
    Ok(0)
}


/// syscall: rt_sigaction
pub fn sys_rt_sigaction(signo: i32, action: *const SigAction, old_action: *mut SigAction) -> SysResult {
    log::debug!(
        "[sys_rt_sigaction]: sig {}, new act ptr {:#x}, old act ptr {:#x}, act size {}",
        signo,
        action as usize,
        old_action as usize,
        core::mem::size_of::<SigAction>()
    );
    if signo < 0 || signo as usize > SIG_NUM {
        info!("[sys_rt_sigaction]: error");
        return Err(SysError::EINVAL);
    }

    let task = current_task().unwrap().clone();
    let sig_manager = task.sig_manager.lock();
    let _sum_guard = SumGuard::new();
    log::debug!("[sys_rt_sigaction]: writing old action");
    if !old_action.is_null() {
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

    log::debug!("[sys_rt_sigaction]: reading new action");
    if !action.is_null() {
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
        log::debug!(
                "[sys_rt_sigaction]: sig {}, set new sig handler {:#x}, sa_mask {:?}, sa_flags: {:#x}, sa_restorer: {:#x}",
                signo,
                new_sigaction.sa.sa_handler as *const usize as usize,
                new_sigaction.sa.sa_mask[0],
                new_sigaction.sa.sa_flags,
                new_sigaction.sa.sa_restorer,
            );
        task.set_sigaction(signo as usize, new_sigaction);
    }
    Ok(0)
}

const SIGBLOCK: i32 = 0;
const SIGUNBLOCK: i32 = 1;
const SIGSETMASK: i32 = 2;

/// syscall: rt_sigprocmask
pub fn sys_rt_sigprocmask(how: i32, set: *const u32, old_set: *mut SigSet) -> SysResult {
    log::debug!("[sys_rt_sigprocmask]: how: {}", how);
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
        return Ok(0);
    }
    let _sum_guard = SumGuard::new();
    
    let new_sig_mask = unsafe { SigSet::from_bits(*set as usize).unwrap() };
    log::debug!(
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
            return Err(SysError::EINVAL);
        }
    };
    Ok(0)
}

/// syscall: rt_sigreturn
pub fn sys_rt_sigreturn() -> SysResult {
    log::debug!("[sys_rt_sigreturn]: into");
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
    Ok(cx.arg_nth(0) as isize)
}

/// suspends execution of the calling thread until one
/// of the signals in set is pending (If one of the signals in set is
/// already pending for the calling thread, sigwaitinfo() will return
/// immediately.)
/// - `set`: Suspend the execution of the process until a signal in `set`
///   that arrives
/// - `info`: If it is not NULL, the buffer that it points to is used to
///   return a structure of type siginfo_t containing information about the
///   signal.
/// - `timeout`: specifies the interval for which the thread is suspended
///   waiting for a signal.
/// On success, sigtimedwait() returns a signal number
pub async fn sys_rt_sigtimedwait(
    set_ptr: usize,
    info_ptr: usize,
    timeout_ptr: usize,
)-> SysResult {
    let task = current_task().unwrap();
    let mut set = unsafe {
        Instruction::set_sum();
        *(set_ptr as *mut SigSet)
    };
    set.remove(SigSet::SIGKILL | SigSet::SIGSTOP);
    let pending_sigs = task.with_mut_sig_manager(|sig_manager| {
        if let Some(si) = sig_manager.check_pending(set) {
            Some(si.si_signo)
        }else {
            sig_manager.wake_sigs = set | SigSet::SIGKILL | SigSet::SIGSTOP;
            None
        }
    });
    if let Some(si) = pending_sigs {
        return Ok(si as isize);
    }
    task.set_interruptable();
    if timeout_ptr == 0 {
        suspend_now().await;
    }else {
        let timeout = unsafe {
            Instruction::set_sum();
            *(timeout_ptr as *const TimeSpec)
        };
        if !timeout.is_valid() {
            return  Err(SysError::EINVAL);
        }
        suspend_timeout(task,timeout.into() ).await;
    }
    task.set_running();
    let si = task.with_mut_sig_manager(|sig_manager| {
        sig_manager.dequeue_expected(set)
    });
    if let Some(si) = si {
        log::warn!("[sys_rt_sigtimedwait] woken by {:#?}",si);
        if info_ptr != 0 {
            unsafe {
                (info_ptr as *mut SigInfo).write(si);
            }
        }    
        return  Ok(si.si_signo as isize);
    }else {
        log::warn!("[sys_rt_sigtimedwait] info_ptr is null, woken by timeout");
        return Err(SysError::EAGAIN);
    }
}