//! io related syscall

use core::{future::Future, mem, pin::Pin, ptr::read, task::{Context, Poll}, time::Duration, usize};

use alloc::{sync::Arc, vec::Vec};
use hal::instruction::{Instruction, InstructionHal};
use log::SetLoggerError;
use virtio_drivers::device::socket::SocketError;

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

#[derive(Debug)]
#[repr(C)]
/// fd set struct
pub struct FdSet {
    /// a set of bits representing the file descriptors status
    pub fds_bits: [usize; FD_SET_LEN],
}
/// fd set size
pub const FD_SET_SIZE: usize = 1024;
/// fd set length
pub const FD_SET_LEN: usize = FD_SET_SIZE / (8 * core::mem::size_of::<usize>());

impl FdSet {
    /// clear all bits in the set
    pub fn clear(&mut self) {
        self.fds_bits.fill(0);
    }
    /// Add the given file descriptor to the collection. Calculate the index and
    /// corresponding bit of the file descriptor in the array, and set the bit
    /// to 1
    pub fn mark_fd(&mut self, fd: usize) {
        if fd >= FD_SET_SIZE {
            return;
        }
        let index = fd / (8 * core::mem::size_of::<usize>());
        let bits = fd % (8 * core::mem::size_of::<usize>());
        let mask = 1 << bits;
        self.fds_bits[index] |= mask;
    }
    /// check if the given fd is set
    pub fn is_set(&self, fd: usize) -> bool {
        let index = fd / (8 * core::mem::size_of::<usize>());
        let bits = fd % (8 * core::mem::size_of::<usize>());
        let mask = 1 << bits;
        self.fds_bits[index] & mask != 0
    }
}

/// monitor multiple file descriptors,
/// waiting until one or more of the file descriptors become "ready"
/// for some class of I/O operation (e.g., input possible). 
pub async fn sys_pselect6(
    nfds: i32,
    readfds_ptr: usize,
    writefds_ptr: usize,
    exceptfds_ptr: usize,
    timeout_ptr: usize,
    sigmask_ptr: usize,
) -> SysResult {
    let task = current_task().unwrap();
    if nfds < 0 {
        return Err(SysError::EINVAL);
    }
    let mut readfds = unsafe {
        if readfds_ptr == 0 {
            None
        }else {
            Instruction::set_sum();
           Some(&mut *(readfds_ptr as *mut FdSet))
        } 
    };
    let mut writefds = unsafe {
        if writefds_ptr == 0 {
            None
        }else {
            Instruction::set_sum();
            Some(&mut *(writefds_ptr as *mut FdSet))
        }
    };
    let mut exceptfds = unsafe {
        if exceptfds_ptr == 0 {
            None
        }else {
            Instruction::set_sum();
            Some(&mut *(exceptfds_ptr as *mut FdSet))
        }
    };
    let timeout: Option<Duration> = unsafe {
        if timeout_ptr == 0 {
            None
        }else {
            Some((*(timeout_ptr as *const TimeSpec)).into())
        }
    };
    log::info!(
        "[sys_pselect]: readfds {:?}, writefds {:?}, exceptfds {:?}, timeout {:?}",
        readfds, writefds, exceptfds, timeout
    );
    let new_mask = if sigmask_ptr == 0 {
        None
    } else {
        Some(unsafe {
            *(sigmask_ptr as *const SigSet)
        })
    };

    let mut polls= Vec::<(usize,PollEvents, Arc<dyn File>)>::with_capacity(nfds as usize);
    for fd in 0..nfds as usize {
        let mut events = PollEvents::empty();
        readfds.as_ref().map(|fds|{
            if fds.is_set(fd) {
                events.insert(PollEvents::IN);
            }
        });
        writefds.as_ref().map(|fds|{
            if fds.is_set(fd) {
                events.insert(PollEvents::OUT);
            }
        });
        if !events.is_empty() {
            let file = task.with_fd_table(|f|f.get_file(fd))?;
            polls.push(
                (fd , events, file)
            );
        }
    }

    // save the old sig mask
    let old_mask = task.with_sig_manager(|m|m.blocked_sigs);
    let mut current_mask = old_mask;
    if let Some(mask) = new_mask {
        task.with_mut_sig_manager(|m| m.blocked_sigs |= mask);
        current_mask |= mask;
    }

    task.set_interruptable();
    task.set_wake_up_sigs(!current_mask);
    let intr_future = IntrBySignalFuture {
        task: task.clone(),
        mask: current_mask
    };
    let pselect_future = PSelectFuture{polls};
    let ret = if let Some(timeout) = timeout {
        match Select2Futures::new(
            TimedTaskFuture::new(timeout,pselect_future),
            intr_future
        ).await {
            SelectOutput::Output1(output1) => match output1 {
                TimedTaskOutput::OK(ret) => ret,
                TimedTaskOutput::TimedOut => {
                    log::info!("[sys_pselect]: timeout!");
                    readfds.as_mut().map(|fds|fds.clear());
                    writefds.as_mut().map(|fds|fds.clear());
                    exceptfds.as_mut().map(|fds|fds.clear());
                    task.set_running();
                    // restore old mask
                    task.with_mut_sig_manager(|m| m.blocked_sigs = old_mask);
                    return Ok(0);
                }
            }
            SelectOutput::Output2(_) => return Err(SysError::EINTR),
        }
    }else {
        match Select2Futures::new(pselect_future, intr_future).await {
            SelectOutput::Output1(ret) => ret,  
            SelectOutput::Output2(_) => return Err(SysError::EINTR),
        }
    };

    readfds.as_mut().map(|fds| fds.clear());
    writefds.as_mut().map(|fds| fds.clear());
    exceptfds.as_mut().map(|fds| fds.clear());

    task.set_running(); 
    // restore old mask
    task.with_mut_sig_manager(|m| m.blocked_sigs = old_mask);
    let mut res = 0;
    for (fd, events) in ret {
        if events.contains(PollEvents::IN) || events.contains(PollEvents::HUP){
            log::info!("[sys_pselect]: fd {} is ready for read", fd);
            readfds.as_mut().map(|fds| fds.mark_fd(fd));
            res += 1;
        }
        if events.contains(PollEvents::OUT) {
            log::info!("[sys_pselect]: fd {} is ready for write", fd);
            writefds.as_mut().map(|fds| fds.mark_fd(fd));
            res += 1;
        }
    }
    Ok(res)
}

/// select future for aysnc select system call
pub struct PSelectFuture {
    polls: Vec<(usize, PollEvents, Arc<dyn File>)>,
}

impl Future for PSelectFuture {
    type Output = Vec<(usize, PollEvents)>;

    /// Return vec of futures that are ready. Return `Poll::Pending` if
    /// no futures are ready.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let mut ret_vec = Vec::with_capacity(this.polls.len());
        for (fd, events, file) in this.polls.iter() {
            let result = unsafe { Pin::new_unchecked(&mut file.poll(*events)).poll(cx) };
            match result {
                Poll::Pending => unreachable!(),
                Poll::Ready(result) => {
                    if !result.is_empty() {
                        ret_vec.push((*fd, result))
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