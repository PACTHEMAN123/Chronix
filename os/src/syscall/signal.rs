//! signal related syscall

use core::task::Waker;
use core::time::Duration;

use alloc::collections::binary_heap::BinaryHeap;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use hal::instruction::{Instruction, InstructionHal};
use hal::println;
use hal::{
    trap::TrapContextHal,
    signal::*,
};
use log::*;
use super::{SysError,SysResult};
use crate::mm::UserPtrRaw;
use crate::sync::mutex::SpinNoIrqLock;
use crate::{processor, timer};
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
    if signo == 0 {
        // If sig is 0, then no signal is sent
        return Ok(0);
    } else if signo < 0 || signo as usize >= SIGRTMAX {
        return Err(SysError::EINVAL);
    }
    let cur_task = current_task().unwrap().clone();
    log::info!("[sys_kill]: task {} sending signo: {} to pid: {}", cur_task.tid(), signo, pid);
    let pgid = cur_task.pgid();
    match pid {
        0 => {
            // sent to every process in the process group of current process
            for process in PROCESS_GROUP_MANAGER
                .get_group(pgid)
                .unwrap()
                .into_iter()
                .map(|inner| inner.upgrade().unwrap())
                .filter(|inner| inner.is_leader())
            {
                process.recv_sigs_process_level(
                    SigInfo {
                        si_signo: signo as usize,
                        si_code: SigInfo::USER,
                        si_pid: Some(cur_task.pid())
                    }
                );
            }
        }
        -1 => {
            // sent to every process which current process has permission ( except init proc )
            //panic!("[sys_kill] unsupport for sending signal to all process");
            TASK_MANAGER.for_each_task(|task|{
                if task.tid() == INITPROC_PID {
                    return;
                }
                if signo != 0 && task.is_leader(){
                    task.recv_sigs_process_level(
                        SigInfo { si_signo: signo as usize, si_code: SigInfo::USER, si_pid: Some(cur_task.pid()) },
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
                .ok_or_else(|| SysError::ESRCH)?
                .into_iter()
                .filter_map(|t| t.upgrade())
            {
                if task.tid() == inner_pid {
                    task.recv_sigs_process_level(SigInfo { si_signo: signo as usize, si_code: SigInfo::USER, si_pid: Some(cur_task.pgid()) });
                }
            }
        }
        _ if pid > 0 => {
            // sent to the process specified with pid
            //assert!(task.gettid() != pid as usize); // should not send to itself
            if let Some(task) = TASK_MANAGER.get_task(pid as usize) {
                if task.is_leader() {
                    task.recv_sigs_process_level(
                        SigInfo { si_signo: signo as usize, si_code: SigInfo::USER, si_pid: Some(cur_task.pid()) },
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
    if signo < 0 || signo as usize > SIGRTMAX || signo as usize == SIGKILL || signo as usize == SIGSTOP {
        info!("[sys_rt_sigaction]: error");
        return Err(SysError::EINVAL);
    }

    let task = current_task().unwrap().clone();
    let sig_manager = task.sig_manager.lock();
    let _sum_guard = SumGuard::new();
    log::debug!("[sys_rt_sigaction]: writing old action");
    if !old_action.is_null() {
        let k_sig_hand = &sig_manager.sig_handler[signo as usize];
        let t = if k_sig_hand.is_user {
            k_sig_hand.sa
        } else {
            let mut sig_hand = k_sig_hand.sa;
            sig_hand.sa_handler = SIG_DFL;
            sig_hand
        };
        UserPtrRaw::new(old_action)
            .ensure_write(&mut task.vm_space.lock())
            .ok_or(SysError::EFAULT)?
            .write(t);
    }
    drop(sig_manager);

    log::debug!("[sys_rt_sigaction]: reading new action");
    if !action.is_null() {
        let mut sig_action = {
            *UserPtrRaw::new(action)
            .ensure_read(&mut task.vm_space.lock())
            .ok_or(SysError::EFAULT)?
            .to_ref()
        };

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
        UserPtrRaw::new(old_set)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EINVAL)?
            .write(sig_manager.blocked_sigs);
        debug!("[sys_rt_sigprocmask] old set: {:?}", sig_manager.blocked_sigs);
    }
    if set as usize == 0 {
        debug!("arg set is null");
        return Ok(0);
    }
    
    let new_sig_mask = SigSet::from_bits(
        *UserPtrRaw::new(set)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EINVAL)?
            .to_ref() as usize
    ).ok_or(SysError::EINVAL)?;
    
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

/// sigpending() returns the set of signals that are pending for
/// delivery to the calling thread (i.e., the signals which have been
/// raised while blocked).  The mask of pending signals is returned in
/// set.
pub fn sys_rt_sigpending(set_ptr: *mut SigSet) -> SysResult {
    let task = current_task().unwrap().clone();
    let sets = task.sig_manager.lock().pending_sigs();
    UserPtrRaw::new(set_ptr)
        .ensure_write(&mut task.vm_space.lock())
        .ok_or(SysError::EFAULT)?
        .write(sets);
    Ok(0)
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
/// 
/// TODOS: the implements seems not accurate
pub async fn sys_rt_sigtimedwait(
    set_ptr: usize,
    info_ptr: usize,
    timeout_ptr: usize,
)-> SysResult {
    let task = current_task().unwrap().clone();
    let mut set = unsafe {
        let _sum_guard = SumGuard::new();
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
        // log::warn!("[sys_rt_sigtimedwait] task {} start to suspend", task.tid());
        suspend_now().await;
    } else {
        let timeout = unsafe {
            let _sum_guard = SumGuard::new();
            *(timeout_ptr as *const TimeSpec)
        };
        log::warn!("[sys_rt_sigtimedwait] task {} set timeout {:?}",task.tid(), timeout);
        if !timeout.is_valid() {
            return  Err(SysError::EINVAL);
        }
        suspend_timeout(current_task().unwrap(), timeout.into()).await;
    }
    task.set_running();
    let si = task.with_mut_sig_manager(|sig_manager| {
        sig_manager.dequeue_expected_one(set)
    });
    if let Some(si) = si {
        log::warn!("[sys_rt_sigtimedwait] task {} woken by {:#?}", task.tid(), si);
        if info_ptr != 0 {
            unsafe {
                (info_ptr as *mut SigInfo).write(si);
            }
        }
        return  Ok(si.si_signo as isize);
    } else {
        log::warn!("[sys_rt_sigtimedwait] info_ptr is null, task {} woken by timeout", task.tid());
        return Err(SysError::EAGAIN);
    }
}

/// syscall: rt_sigsuspend
/// temporarily replaces the signal mask of the calling thread with the mask
/// given by mask and then suspends the thread until delivery of a signal
/// whose action is to invoke a signal handler or to terminate a process
///
/// If the signal terminates the process, then sigsuspend() does not return.
/// If the signal is caught, then sigsuspend() returns after the signal
/// handler returns, and the signal mask is restored to the state before
/// the call to sigsuspend().
///
/// It is not possible to block SIGKILL or SIGSTOP; specifying these signals
/// in mask, has no effect on the thread's signal mask.
/// sigsuspend() always returns -1, normally with the error EINTR.
pub async fn sys_rt_sigsuspend(mask_ptr: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let mut mask = unsafe {
        Instruction::set_sum();
        *(mask_ptr as *const SigSet)
    };
    log::info!("[sys_rt_sigsuspend] task {} use mask {:?} suspend", task.tid(), mask);
    mask.remove(SigSet::SIGSTOP | SigSet::SIGKILL);
    // replace the signal mask using given mask
    let mut oldmask = SigSet::empty();
    task.with_mut_sig_manager(|sig_manager| {
        oldmask = sig_manager.blocked_sigs;
        sig_manager.blocked_sigs = mask
    });
    // TODOS: is the logic here correct?
    let invoke_sigs = task.with_sig_manager(|s| s.user_define_sets());
    task.with_mut_sig_manager(|sig_manager| {
        if sig_manager.check_pending_flag(!mask | invoke_sigs) {
            Err(SysError::EINTR)
        } else {
            sig_manager.wake_sigs = !mask | invoke_sigs;
            Ok(())
        }
    })?;
    task.set_interruptable();
    suspend_now().await;
    // restore mask
    task.with_mut_sig_manager(|sig_manager| {
        sig_manager.blocked_sigs = oldmask
    });
    task.set_running();
    Err(SysError::EINTR)
}


/// tkill() is an obsolete predecessor to tgkill().  It allows only
///        the target thread ID to be specified, which may result in the
///        wrong thread being signaled if a thread terminates and its thread
///        ID is recycled.  Avoid using this system call.
pub fn sys_tkill(tid: isize, sig: i32) -> SysResult {
    info!("[sys_tkill] {} {}", tid, sig);
    if (sig < 0) || sig as usize >= SIGRTMAX || tid < 0{
        return Err(SysError::EINVAL);
    }
    let cur_task = current_task().unwrap();
    let task = TASK_MANAGER.get_task(tid as usize)
        .ok_or(SysError::ESRCH)?;
    task.recv_sigs(
        SigInfo {
            si_signo: sig as usize,
            si_code: SigInfo::TKILL,
            si_pid: Some(cur_task.pid()),
        }
    );
    Ok(0)
}

/// sends the signal sig to the thread with the thread ID tid
///        in the thread group tgid.  (By contrast, kill(2) can be used to
///        send a signal only to a process (i.e., thread group) as a whole,
///        and the signal will be delivered to an arbitrary thread within
///        that process.)
pub fn sys_tgkill(tgid: isize, tid: isize, signo: i32) -> SysResult {
    info!("[sys_tgkill] {} {} {}", tgid, tid, signo);
    if (signo < 0) || signo as usize >= SIGRTMAX || tid < 0{
        return Err(SysError::EINVAL);
    }
    if tgid < 0 || tid < 0 {
        return Err(SysError::EINVAL);
    }
    let cur_task = current_task().unwrap();
    let task = TASK_MANAGER.get_task(tgid as usize).ok_or(SysError::ESRCH)?;
    if task.is_leader() {
        task.with_mut_thread_group(|thread_group| -> SysResult {
            for thread in thread_group.iter() {
                if thread.tid() == tid as usize {
                    thread.recv_sigs(SigInfo { si_signo: signo as usize, si_code: SigInfo::TKILL, si_pid: Some(cur_task.pid())});
                    return Ok(0)
                }
            }
            return Err(SysError::ESRCH);
        })
    }else {
        return Err(SysError::ESRCH);
    }
}