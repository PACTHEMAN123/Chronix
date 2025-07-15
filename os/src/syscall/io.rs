//! io related syscall

use core::{future::Future, mem, pin::Pin, ptr::read, task::{Context, Poll}, time::Duration, usize};

use alloc::{sync::Arc, vec::Vec};
use hal::instruction::{Instruction, InstructionHal};
use log::SetLoggerError;
use smoltcp::time;
use virtio_drivers::device::socket::SocketError;

use crate::{fs::vfs::{file::PollEvents, File}, mm::{UserPtrRaw, UserSliceRaw}, signal::SigSet, task::{current_task, signal::IntrBySignalFuture}, timer::{ffi::TimeSpec, timed_task::{PendingFuture, TimedTaskFuture, TimedTaskOutput}}, utils::{Select2Futures, SelectOutput}};

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
    // let raw_fds: &mut [PollFd] = unsafe {
    //     Instruction::set_sum();
    //     core::slice::from_raw_parts_mut(fds as *mut PollFd, nfds)
    // };
    // log::info!("fds: {fds}, nfds: {nfds}, timeout_ts: {timeout_ts}, sigmask: {sigmask}");
    if (nfds as i32) < 0 {
        return Err(SysError::EINVAL);
    }
    let raw_fds = UserSliceRaw::new(fds as *mut PollFd, nfds)
    .ensure_write(&mut task.get_vm_space().lock())
    .ok_or(SysError::EFAULT)?;
    let raw_fds = raw_fds.to_mut();
    let mut poll_fds: Vec<PollFd> = Vec::new();
    poll_fds.extend_from_slice(raw_fds);

    let timeout = if timeout_ts == 0 {
        None
    } else {
        let ret = *UserPtrRaw::new(timeout_ts as *const TimeSpec)
            .ensure_read(&mut  task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
            .to_ref();
        if !ret.is_valid(){
            return Err(SysError::EINVAL);
        }
        Some(ret)
    };

    let new_mask = if sigmask == 0 {
        None
    } else {
        Some(
            *UserPtrRaw::new(sigmask as *const SigSet)
            .ensure_read(&mut  task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
            .to_ref()
        )
    };

    // put the file in the vec of polling futures
    let mut polls = Vec::<(PollEvents, Arc<dyn File>)>::with_capacity(nfds);
    for (_i, poll_fd) in poll_fds.iter_mut().enumerate() {
        let fd = poll_fd.fd as usize;
        let events = poll_fd.events;
        // let file = task.with_fd_table(|t| t.get_file(fd))?;
        match task.with_fd_table(|t| t.get_file(fd)) {
            Ok(file) => {
                polls.push((events, file));
            }
            _ => {
                poll_fd.revents |= PollEvents::INVAL;
            }
        }
    }

    // save the old sig mask
    let old_mask = task.sig_manager.lock().blocked_sigs;
    let mut current_mask = old_mask;
    if let Some(mask) = new_mask {
        task.sig_manager.lock().blocked_sigs |= mask;
        current_mask |= mask;
    }

    if nfds == 0 {
        task.set_interruptable();
        task.set_wake_up_sigs(!current_mask);

        if let Some(timeout) = timeout {
            let sleep_future = TimedTaskFuture::new(
                timeout.into(),
                PendingFuture{}
            );
            let intr_future = IntrBySignalFuture {
                task: task.clone(),
                mask: current_mask,
            };

            let result = Select2Futures::new(sleep_future, intr_future).await;
            task.set_running();
            task.sig_manager.lock().blocked_sigs = old_mask;
            match result {
                SelectOutput::Output1(_) => {
                    raw_fds.copy_from_slice(&poll_fds);
                    return Ok(poll_fds.iter().filter(|f| !f.revents.is_empty()).count() as isize);

                }
                SelectOutput::Output2(_) => {
                    return Err(SysError::EINTR);
                }
            }
        }else {
            let intr_future = IntrBySignalFuture {
                task: task.clone(),
                mask: current_mask,
            };
            match intr_future.await {
                _ => return Err(SysError::EINTR),
            }
        }
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

#[derive(Debug, Copy, Clone)]
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
        let idx = fd / 64;
        let bit = fd % 64;
        let mask = 1 << bit;
        self.fds_bits[idx] |= mask;
    }
    /// check if the given fd is set
    pub fn is_set(&self, fd: usize) -> bool {
        let idx = fd / 64;
        let bit = fd % 64;
        let mask = 1 << bit;
        self.fds_bits[idx] & mask != 0
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
    if nfds < 0 {
        return Err(SysError::EINVAL);
    }
    let task = current_task().unwrap().clone();
    let mut readfds = {
        if readfds_ptr == 0 {
            None
        }else {
            let raw_fds = UserPtrRaw::new(readfds_ptr as *mut FdSet)
                    .ensure_read(&mut task.get_vm_space().lock())
                    .ok_or(SysError::EFAULT)?;
            Some(
                *raw_fds.to_ref()
            )
        } 
    };
    let mut writefds = {
        if writefds_ptr == 0 {
            None
        }else {
            Some(*UserPtrRaw::new(writefds_ptr as *mut FdSet)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
            .to_ref())
        }
    };
    let mut exceptfds = {
        if exceptfds_ptr == 0 {
            None
        }else {
            Some(*UserPtrRaw::new(exceptfds_ptr as *mut FdSet)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
            .to_ref())
        }
    };
    let timeout: Option<Duration> = {
        if timeout_ptr == 0 {
            None
        }else {
            let ret = *UserPtrRaw::new(timeout_ptr as *const TimeSpec)
            .ensure_read(&mut  task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
            .to_ref();
            if !ret.is_valid(){
                return Err(SysError::EINVAL);
            }
            Some(ret.into())
        }
    };
    if let Some(inner_timeout) = timeout {
        log::info!("timeout: {:?}",inner_timeout);
    }
    // log::info!(
    //     "[sys_pselect]: readfds {:?}, writefds {:?}, exceptfds {:?}, timeout {:?}",
    //     readfds, writefds, exceptfds, timeout
    // );
    let new_mask = if sigmask_ptr == 0 {
        None
    } else {
        Some(*UserPtrRaw::new(sigmask_ptr as *const SigSet)
            .ensure_read(&mut  task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
            .to_ref()
        )
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
    let mut prev_mask = None; 
    if let Some(mask) = new_mask {
        task.with_mut_sig_manager(|sig_manager| {
            prev_mask = Some(sig_manager.blocked_sigs);
            sig_manager.blocked_sigs |= mask;
        })
    }
    task.set_interruptable();
    task.set_wake_up_sigs(task.with_sig_manager(|m| !m.blocked_sigs));
    let intr_future = IntrBySignalFuture {
        task: task.clone(),
        mask: task.with_sig_manager(|m|m.blocked_sigs),
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
                    // log::info!("[sys_pselect]: timeout!");
                    readfds.as_mut().map(|fds|fds.clear());
                    writefds.as_mut().map(|fds|fds.clear());
                    exceptfds.as_mut().map(|fds|fds.clear());
                    task.set_running();
                    // restore old mask
                    if let Some(mask) = prev_mask {
                        task.with_mut_sig_manager(|m|m.blocked_sigs = mask);
                    }
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
    if let Some(mask) = prev_mask {
        task.with_mut_sig_manager(|m| m.blocked_sigs = mask);
    }
    let mut res = 0;
    for (fd, events) in ret {
        if events.contains(PollEvents::IN) || events.contains(PollEvents::HUP){
            // log::info!("[sys_pselect]: fd {} is ready for read", fd);
            readfds.as_mut().map(|fds| fds.mark_fd(fd));
            res += 1;
        }
        if events.contains(PollEvents::OUT) {
            // log::info!("[sys_pselect]: fd {} is ready for write", fd);
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