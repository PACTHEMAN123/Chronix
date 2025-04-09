//! io related syscall

use core::{future::Future, pin::Pin, task::Poll, time::Duration};

use alloc::{sync::Arc, vec::Vec};
use hal::instruction::{Instruction, InstructionHal};

use crate::{fs::vfs::{file::PollEvents, File}, signal::SigSet, task::{current_task, signal::IntrBySignalFuture}, timer::{ffi::TimeSpec, timed_task::{TimedTaskFuture, TimedTaskOutput}}, utils::{Select2Futures, SelectOutput}};

use super::{SysError, SysResult};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[allow(missing_docs)]
pub struct PollFd {
    /// file descriptor
    fd: i32,
    /// requested events    
    events: PollEvents,
    /// returned events
    revents: PollEvents,
}

/// future use to poll the files
pub struct PPollFuture {
    polls: Vec<(PollEvents, Arc<dyn File>)>,
}

impl Future for PPollFuture {
    type Output = Vec<(usize, PollEvents)>;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let mut ret_vec = Vec::new();
        for (i, (events, file)) in this.polls.iter().enumerate() {
            // try to poll every file in the polls vec
            let res = unsafe { Pin::new_unchecked(&mut file.poll(*events)).poll(cx) };
            match res {
                Poll::Pending => unreachable!(),
                Poll::Ready(res) => {
                    if !res.is_empty() {
                        ret_vec.push((i, res));
                    }
                }
            }
        }
        if ret_vec.len() > 0 {
            Poll::Ready(ret_vec)
        } else {
            Poll::Pending
        }
    }
}

/// syscall: ppoll
/// it waits for one of a set of file descriptors to become ready to perform I/O.
pub async fn sys_ppoll(fds: usize, nfds: usize, timeout_ts: usize, sigmask: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let raw_fds: &mut [PollFd] = unsafe {
        Instruction::set_sum();
        core::slice::from_raw_parts_mut(fds as *mut PollFd, nfds)
    };
    let mut poll_fds: Vec<PollFd> = Vec::new();
    poll_fds.extend_from_slice(raw_fds);

    let timeout = if timeout_ts == 0 {
        None
    } else {
        Some(unsafe {
            *(timeout_ts as *const TimeSpec)
        })
    };

    let new_mask = if sigmask == 0 {
        None
    } else {
        Some(unsafe {
            *(sigmask as *const SigSet)
        })
    };

    // put the file in the vec of polling futures
    let mut polls = Vec::<(PollEvents, Arc<dyn File>)>::with_capacity(nfds);
    for poll_fd in poll_fds.iter() {
        let fd = poll_fd.fd as usize;
        let events = poll_fd.events;
        let file = task.with_fd_table(|t| t.get_file(fd))?;
        polls.push((events, file));
    }

    // save the old sig mask
    let old_mask = task.sig_manager.lock().blocked_sigs;
    let mut current_mask = old_mask;
    if let Some(mask) = new_mask {
        task.sig_manager.lock().blocked_sigs |= mask;
        current_mask |= mask;
    }

    let poll_future = PPollFuture { polls };
    task.set_interruptable();
    task.set_wake_up_sigs(!current_mask);

    let ret_vec = if let Some(timeout) = timeout {
        // need to set a timer
        match TimedTaskFuture::new(timeout.into(), poll_future).await {
            TimedTaskOutput::OK(ret_vec) => ret_vec,
            TimedTaskOutput::TimedOut => {
                log::info!("timeout!");
                return Ok(0);
            }
        }
    } else {
        let intr_future = IntrBySignalFuture {
            task: task.clone(),
            mask: current_mask,
        };
        match Select2Futures::new(poll_future, intr_future).await {
            SelectOutput::Output1(ret_vec) => ret_vec,
            SelectOutput::Output2(_) => return Err(SysError::EINTR),
        }
    };
    task.set_running();
    let ret = ret_vec.len();
    for (i, result) in ret_vec {
        poll_fds[i].revents |= result;
    }
    raw_fds.copy_from_slice(&poll_fds);

    // restore the sig mask
    task.sig_manager.lock().blocked_sigs = old_mask;
    Ok(ret as isize)
}