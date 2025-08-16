//! io related syscall

use core::{cmp, future::Future, mem, num::NonZeroI64, pin::Pin, ptr::read, sync::atomic::AtomicUsize, task::{Context, Poll}, time::Duration, usize};
use alloc::{boxed::Box, string::ToString};
use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use async_trait::async_trait;
use hal::instruction::{Instruction, InstructionHal};
use log::SetLoggerError;
use lwext4_rust::bindings::PRIX16;
use smoltcp::time;
use spin::{Lazy, Mutex};
use universal_hash::generic_array::functional;
use virtio_drivers::device::socket::SocketError;
use xmas_elf::reader;

use crate::{fs::{vfs::{file::PollEvents, File, FileInner}, OpenFlags}, mm::{UserPtrRaw, UserSliceRaw}, signal::{msg_queue::{MessageQueue, MqAttr, MqError, NotifyRegistration, Sigevent, MQ_FLAG_NONBLOCK, SIGEV_NONE, SIGEV_SIGNAL}, SigSet, SIGKILL}, sync::mutex::SpinNoIrqLock, task::{current_task, fs::{FdFlags, FdInfo}, signal::IntrBySignalFuture}, timer::{ffi::TimeSpec, get_current_time_duration, timed_task::{PendingFuture, TimedTaskFuture, TimedTaskOutput}}, utils::{suspend_now, user_path_to_string, Select2Futures, SelectOutput}};
use crate::fs::tmpfs::dentry::TmpDentry;
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
    let task = current_task().unwrap();
    let mut readfds_w = {
        if readfds_ptr == 0 {
            None
        }else {
            //     Instruction::set_sum();
            //    Some(&mut *(readfds_ptr as *mut FdSet))
            Some(UserPtrRaw::new(readfds_ptr as *mut FdSet)
               .ensure_write(&mut task.get_vm_space().lock())
               .ok_or(SysError::EFAULT)?
            )
        } 
    };
    let readfds_r = {
        if readfds_ptr == 0 {
            None
        }else {
            //     Instruction::set_sum();
            //    Some(&mut *(readfds_ptr as *mut FdSet))
            Some(UserPtrRaw::new(readfds_ptr as *const FdSet)
               .ensure_read(&mut task.get_vm_space().lock())
               .ok_or(SysError::EFAULT)?
            )
        } 
    };
    let mut writefds_w = {
        if writefds_ptr == 0 {
            None
        }else {
            // Instruction::set_sum();
            // Some(&mut *(writefds_ptr as *mut FdSet))
            Some(UserPtrRaw::new(writefds_ptr as *mut FdSet)
               .ensure_write(&mut task.get_vm_space().lock())
               .ok_or(SysError::EFAULT)?
            )
        }
    };
    let writefds_r = {
        if writefds_ptr == 0 {
            None
        }else {
            // Instruction::set_sum();
            // Some(&mut *(writefds_ptr as *const FdSet))
            Some(UserPtrRaw::new(writefds_ptr as *const FdSet)
               .ensure_read(&mut task.get_vm_space().lock())
               .ok_or(SysError::EFAULT)?
            )
        }
    };
    let mut exceptfds_w = {
        if exceptfds_ptr == 0 {
            None
        }else {
            // Instruction::set_sum();
            // Some(&mut *(exceptfds_ptr as *mut FdSet))
            Some(UserPtrRaw::new(exceptfds_ptr as *mut FdSet)
               .ensure_write(&mut task.get_vm_space().lock())
               .ok_or(SysError::EFAULT)?
            )
        }
    };

    let timeout = {
        if timeout_ptr == 0 {
            None
        }else {
            // Instruction::set_sum();
            // Some((*(timeout_ptr as *const TimeSpec)).into())
            Some(UserPtrRaw::new(timeout_ptr as *const TimeSpec)
                .ensure_read(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?
            )
        }
    };
    // log::info!(
    //     "[sys_pselect]: readfds {:?}, writefds {:?}, exceptfds {:?}, timeout {:?}",
    //     readfds, writefds, exceptfds, timeout
    // );
    let new_mask = if sigmask_ptr == 0 {
        None
    } else {
        // unsafe {
        //     Instruction::set_sum();
        //     Some(*(sigmask_ptr as *const SigSet))
        // }
        Some(
            UserPtrRaw::new(sigmask_ptr as *const SigSet)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
        )
    };

    let mut polls= Vec::<(usize,PollEvents, Arc<dyn File>)>::with_capacity(nfds as usize);
    for fd in 0..nfds as usize {
        let mut events = PollEvents::empty();
        readfds_r.as_ref().map(|fds|{
            if fds.to_ref().is_set(fd) {
                events.insert(PollEvents::IN);
            }
        });
        writefds_r.as_ref().map(|fds|{
            if fds.to_ref().is_set(fd) {
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
            sig_manager.blocked_sigs |= *mask.to_ref();
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
        if !(*timeout.to_ref()).is_valid(){
            return Err(SysError::EINVAL);
        }
        match Select2Futures::new(
            TimedTaskFuture::new((*timeout.to_ref()).into(), pselect_future),
            intr_future
        ).await {
            SelectOutput::Output1(output1) => match output1 {
                TimedTaskOutput::OK(ret) => ret,
                TimedTaskOutput::TimedOut => {
                    // log::info!("[sys_pselect]: timeout!");
                    readfds_w.as_mut().map(|fds|fds.to_mut().clear());
                    writefds_w.as_mut().map(|fds|fds.to_mut().clear());
                    exceptfds_w.as_mut().map(|fds|fds.to_mut().clear());
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

    readfds_w.as_mut().map(|fds| fds.to_mut().clear());
    writefds_w.as_mut().map(|fds| fds.to_mut().clear());
    exceptfds_w.as_mut().map(|fds| fds.to_mut().clear());

    task.set_running(); 
    // restore old mask
    if let Some(mask) = prev_mask {
        task.with_mut_sig_manager(|m| m.blocked_sigs = mask);
    }
    let mut res = 0;
    for (fd, events) in ret {
        if events.contains(PollEvents::IN) || events.contains(PollEvents::HUP){
            // log::info!("[sys_pselect]: fd {} is ready for read", fd);
            readfds_w.as_mut().map(|fds| fds.to_mut().mark_fd(fd));
            res += 1;
        }
        if events.contains(PollEvents::OUT) {
            // log::info!("[sys_pselect]: fd {} is ready for write", fd);
            writefds_w.as_mut().map(|fds| fds.to_mut().mark_fd(fd));
            res += 1;
        }
    }
    log::info!("select finish");
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


bitflags! {
    // define in <uapi/linux/eventpoll.h>
    pub struct EPollEvents: u32 {
        const EPOLLIN	    = 0x00000001;
        const EPOLLPRI	    = 0x00000002;
        const EPOLLOUT	    = 0x00000004;
        const EPOLLERR	    = 0x00000008;
        const EPOLLHUP	    = 0x00000010;
        const EPOLLNVAL	    = 0x00000020;
        const EPOLLRDNORM	= 0x00000040;
        const EPOLLRDBAND	= 0x00000080;
        const EPOLLWRNORM	= 0x00000100;
        const EPOLLWRBAND	= 0x00000200;
        const EPOLLMSG	    = 0x00000400;
        const EPOLLRDHUP	= 0x00002000;
        // input flags
        const EPOLL_URING_WAKE  = 0x08000000;
        const EPOLLEXCLUSIVE    = 0x10000000;
        const EPOLLWAKEUP       = 0x20000000;
        const EPOLLONESHOT      = 0x40000000;
        const EPOLLET           = 0x80000000;
    }
}

impl EPollEvents {
    // remove the input flags
    pub fn remove_input(&mut self) -> Self {
        let mut ret = *self;
        ret.remove(
            EPollEvents::EPOLL_URING_WAKE |
            EPollEvents::EPOLLEXCLUSIVE |
            EPollEvents::EPOLLWAKEUP |
            EPollEvents::EPOLLONESHOT |
            EPollEvents::EPOLLET
        );
        ret
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EPollEvent {
    events: EPollEvents,
    data: usize,
}

pub struct EPollFd {
    file:  Arc<dyn File>,
    event: EPollEvent,
}

// for the epoll machamic
pub struct EPollInstance {
    interest: SpinNoIrqLock<BTreeMap<usize, EPollFd>>,
    ready: SpinNoIrqLock<Vec<(usize, EPollEvent)>>,
    file_inner: FileInner, 
}

impl EPollInstance  {
    pub fn new(fd_flags: FdFlags) -> Self {
        let o_flags = match fd_flags {
            FdFlags::CLOEXEC => OpenFlags::O_CLOEXEC,
            _  => OpenFlags::empty(),
        };

        Self { 
            interest: SpinNoIrqLock::new(BTreeMap::new()),
            ready: SpinNoIrqLock::new(Vec::new()),
            file_inner: FileInner { 
                dentry: TmpDentry::new("", None),
                offset: AtomicUsize::new(0),
                flags: SpinNoIrqLock::new(o_flags)
            }
        }
    }

    pub fn add(&self, fd: usize, event: EPollEvent, file: Arc<dyn File>) -> Result<(), SysError> {
        let mut list = self.interest.lock();
        if list.contains_key(&fd) {
            return Err(SysError::EEXIST)
        }
        list.insert(fd, EPollFd { file, event });
        Ok(()) 
    }

    pub fn remove(&self, fd: usize) -> Result<(), SysError> {
        let mut list = self.interest.lock();
        if !list.contains_key(&fd) {
            return Err(SysError::ENOENT)
        }
        list.remove(&fd);
        Ok(())
    }

    pub fn modify(&self, fd: usize, event: EPollEvent) -> Result<(), SysError> {
        let mut list = self.interest.lock();
        if !list.contains_key(&fd) {
            return Err(SysError::ENOENT)
        }
        if let Some(epoll_fd) = list.get_mut(&fd) {
            epoll_fd.event = event;
        } else {
            return Err(SysError::ENOENT)
        }
        Ok(())
    }
    /// try to fill the event vec as much as possible
    pub fn get_ready(&self, event_vec: &mut [EPollEvent]) -> usize {
        let ready_list = self.ready.lock();
        let get_size = cmp::min(event_vec.len(), ready_list.len());
        for i in 0..get_size {
            event_vec[i] = ready_list[i].1;
        }
        get_size
    }
}

unsafe impl Send for EPollInstance {}
unsafe impl Sync for EPollInstance {}

#[async_trait]
impl File for EPollInstance {
    async fn read(&self, _buf: &mut [u8]) -> Result<usize, SysError> {
        Err(SysError::EBADF)
    }
    async fn write(&self, _buf: &[u8]) -> Result<usize, SysError> {
        Err(SysError::EBADF)
    }
    async fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> Result<usize, SysError> {
        Err(SysError::EBADF)
    }
    async fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, SysError> {
        let inode = self.dentry().unwrap().inode().unwrap();
        let size = inode.cache_write_at(offset, buf).unwrap();
        Ok(size)
    }
    fn readable(&self) -> bool { false }
    fn writable(&self) -> bool { false }
    fn file_inner(&self) -> &FileInner { &self.file_inner }
}


pub fn sys_epoll_create(size: isize) -> SysResult {
    if size < 0 {
        return Err(SysError::EINVAL)
    }
    let task = current_task().unwrap().clone();
    let fd = task.with_mut_fd_table(|t| t.alloc_fd())?;
    let epoll_inst = Arc::new(EPollInstance::new(FdFlags::empty()));
    let fd_info = FdInfo {
        file: epoll_inst,
        flags: FdFlags::empty(),
    };
    task.with_mut_fd_table(|t| t.put_file(fd, fd_info))?;
    log::info!("task {} get epoll instance fd {}", task.tid(), fd);
    Ok(fd as isize)
}

pub fn sys_epoll_create1(flags: usize) -> SysResult {
    const EPOLL_CLOEXEC: usize = OpenFlags::O_CLOEXEC.bits() as usize;
    match flags {
        0 => sys_epoll_create(1),
        EPOLL_CLOEXEC => {
            let task = current_task().unwrap().clone();
            let fd = task.with_mut_fd_table(|t| t.alloc_fd())?;
            let epoll_inst = Arc::new(EPollInstance::new(FdFlags::CLOEXEC));
            let fd_info = FdInfo {
                file: epoll_inst,
                flags: FdFlags::CLOEXEC,
            };
            task.with_mut_fd_table(|t| t.put_file(fd, fd_info))?;
            log::info!("task {} get epoll instance fd {}", task.tid(), fd);
            Ok(fd as isize)
        }
        _ => Err(SysError::EINVAL)
    }
}

const EPOLL_CTL_ADD: usize = 1;
const EPOLL_CTL_DEL: usize = 2;
const EPOLL_CTL_MOD: usize = 3;


pub fn sys_epoll_ctl(epfd: usize, op: usize, fd: usize, event_ptr: usize) -> SysResult {
    if fd == epfd || event_ptr == 0 {
        return Err(SysError::EINVAL)
    }
    let task = current_task().unwrap().clone();
    let epoll_inst = task.with_fd_table(|t| t.get_file(epfd))?;
    let epoll_inst = epoll_inst.downcast_ref::<EPollInstance>().ok_or(SysError::EINVAL)?;
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let event_ptr = UserPtrRaw::new(event_ptr as *const EPollEvent)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let event = *event_ptr.to_ref();
    match op {
        EPOLL_CTL_ADD => {
            epoll_inst.add(fd, event, file)?;
        }
        EPOLL_CTL_DEL => {
            epoll_inst.remove(fd)?;
        }
        EPOLL_CTL_MOD => {
            epoll_inst.modify(fd, event)?;
        }
        _ => return Err(SysError::EINVAL)
    }
    Ok(0)
}

/// poll the fds from epoll instance
/// poll from the interest list,
/// once the fd is finish, place it in ready list 
pub struct EPollFuture {
    epoll_inst: Arc<EPollInstance>,
}

impl Future for EPollFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for (fd, epoll_fd) in self.epoll_inst.interest.lock().iter() {
            let file = epoll_fd.file.clone();
            let events = epoll_fd.event.events;
            let r = unsafe {
                Pin::new_unchecked(&mut file.epoll(events))
                .poll(cx)
            };
            match r {
                Poll::Pending => unreachable!(),
                Poll::Ready(result) => {
                    let mut ret_event = epoll_fd.event;
                    ret_event.events = result;
                    self.epoll_inst.ready.lock().push((*fd, ret_event));
                }
            }
        }
        Poll::Ready(())
    }
}

pub async fn sys_epoll_pwait(epfd: usize, events_ptr: usize, maxenvets: usize, timeout: usize, sigmask_ptr: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let ep_inst_file = task.with_fd_table(|t| t.get_file(epfd))?;
    let ep_inst = match ep_inst_file.downcast_arc::<EPollInstance>() {
        Ok(inst) => inst,
        _ => return Err(SysError::EINVAL)
    };
    if (maxenvets as isize) <= 0 {
        return Err(SysError::EINVAL)
    }
    let events = UserSliceRaw::new(events_ptr as *mut EPollEvent, maxenvets)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;

    // check the ready list, if not empty, return immediately
    match ep_inst.clone().get_ready(events.to_mut()) {
        0 => {},
        nfds => return Ok(nfds as isize)
    }
     
    // no ready events, start to wait
    let timeout = match timeout {
        0 => return Ok(0), // return immediately, even no ready event
        -1 => None,
        t => Some(TimeSpec::from_ms(t)),
    };

    let old_sigmask = task.sig_manager.lock().get_sigmask();
    let mut new_sigmask = old_sigmask;
    if sigmask_ptr != 0 {
        let sigmask_ptr = UserPtrRaw::new(sigmask_ptr as *const SigSet)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
        new_sigmask = *sigmask_ptr.to_ref();
    }
    task.sig_manager.lock().set_sigmask(new_sigmask);
    
    let intr_future = IntrBySignalFuture { task: task.clone(), mask: new_sigmask };
    let epoll_future = EPollFuture { epoll_inst: ep_inst.clone() };

    task.set_interruptable();
    task.set_wake_up_sigs(!new_sigmask);

    if let Some(timeout) = timeout {
        // select from intr and timedtask and epoll event
        let timed_future = TimedTaskFuture::new(
            timeout.into(), 
            epoll_future
        );
        let sel_res = Select2Futures::new(timed_future, intr_future).await;
        // wake up, should restore states before return to user
        task.set_running();
        task.sig_manager.lock().set_sigmask(old_sigmask);
        match sel_res {
            SelectOutput::Output1(_) => {
                let nfds = ep_inst.get_ready(events.to_mut());
                return Ok(nfds as isize)
            }
            SelectOutput::Output2(_) => return Err(SysError::EINTR)
        }
    } else {
        // select from intr and epoll event
        let sel_res = Select2Futures::new(epoll_future, intr_future).await;
        task.set_running();
        task.sig_manager.lock().set_sigmask(old_sigmask);
        match sel_res {
            SelectOutput::Output1(_) => {
                let nfds = ep_inst.get_ready(events.to_mut());
                return Ok(nfds as isize)
            }
            SelectOutput::Output2(_) => return Err(SysError::EINTR)
        }
    }
}

/// msg queue concernign
type MqName = alloc::string::String;

struct MqRegistry {
    queues: BTreeMap<MqName, Arc<MessageQueue>>,
}

static MQ_REGISTRY: Lazy<Mutex<MqRegistry>> = Lazy::new(|| Mutex::new(MqRegistry {
    queues: BTreeMap::new(),
}));


pub fn sys_mq_open(name_ptr: usize, oflag: i32, _mode: u32, attr_ptr: usize) -> SysResult {
    let task = current_task().unwrap();
    let current_uid = task.euid();
    log::info!("curent_uid: {}", current_uid);
    let name_user_ptr = UserPtrRaw::new(name_ptr as *const u8);

    let name = user_path_to_string(name_user_ptr, &mut task.get_vm_space().lock())?;
    log::info!("paramter name: {}", name);
    if name.is_empty() || name[1..].contains('/'){ 
        return Err(SysError::EINVAL);
    }
    log::info!("name start ok");
    const NAME_MAX: usize = 255;
    if name.len() > NAME_MAX {
        return Err(SysError::ENAMETOOLONG);
    }
    log::info!("name length ok");
    // log::info!("[mq] open: name='{}', oflag={}", name, oflag);

    let mut registry = MQ_REGISTRY.lock();
    let queue_exists = registry.queues.contains_key(&name);
    let create = (oflag & OpenFlags::O_CREAT.bits()) != 0;

    let queue = if create {
        log::info!("have create flag");
        if queue_exists {
            let queue = registry.queues.get(&name).unwrap().clone();
            if current_uid != 0 && queue.owner_uid != current_uid {
                log::info!("current_uid: {}, queue.owner_uid: {}", current_uid, queue.owner_uid);
                return Err(SysError::EACCES);
            }
            // --- 创建新队列的逻辑 ---
            if oflag & OpenFlags::O_EXCL.bits() != 0 {
                log::warn!("[mq] open: queue '{}' already exists, have oEXL flag", name);
                return Err(SysError::EEXIST); // 队列已存在且指定了 O_EXCL
            }
            // 直接打开现有队列
            queue
        }
         else {
            // 创建新队列
            let attr = if attr_ptr != 0 {
                let attr_user = UserPtrRaw::new(attr_ptr as *const MqAttr)
                    .ensure_read(&mut task.get_vm_space().lock())
                    .ok_or(SysError::EFAULT)?;
                *attr_user.to_ref()
            } else {
                // 如果 O_CREAT 但未提供 attr，使用默认值
                MqAttr { mq_flags: 0, mq_maxmsg: 10, mq_msgsize: 256 }
            };
            
            // 将 O_NONBLOCK 标志从 oflag 同步到队列属性中
            let mut final_attr = attr;
            if (oflag & OpenFlags::O_NONBLOCK.bits()) != 0 {
                final_attr.mq_flags |= OpenFlags::O_NONBLOCK.bits() as i64;
            }
            if !final_attr.is_valid(){
                return Err(SysError::EINVAL);
            }
            log::info!("final attr ok");
            let new_queue = Arc::new(MessageQueue::new(final_attr, current_uid));
            registry.queues.insert(name, new_queue.clone());
            new_queue
        }
    } else {
        if !queue_exists {
            log::warn!("have no create flag, but queue '{}' not exists", name);
            return Err(SysError::ENOENT); // 队列不存在且未指定 O_CREAT
        }
        let queue = registry.queues.get(&name).unwrap().clone();
        let queue_owner_uid = queue.owner_uid;
        if current_uid != 0 && current_uid != queue_owner_uid {
            log::info!("current_uid: {}, queue.owner_uid: {}", current_uid, queue_owner_uid);
            return Err(SysError::EACCES);
        }
        queue
    };
    let fd = task.with_mut_fd_table(|table| -> SysResult{
        // Arc<MessageQueue> 实现了 File trait，可以被当作 Arc<dyn File>
        let fd = table.alloc_fd()?;
        table.put_file(fd, FdInfo {
            file: queue.clone(),
            flags: FdFlags::empty(),
        })?;
        Ok(fd as isize)
    })?;
    Ok(fd as isize)
}


// ------------------------ sys_mq_unlink ------------------------
pub fn sys_mq_unlink(name_ptr: usize) -> SysResult {
    let task = current_task().unwrap();
   let name_user_ptr = UserPtrRaw::new(name_ptr as *const u8);

    let name = user_path_to_string(name_user_ptr, &mut task.get_vm_space().lock())?;
    const NAME_MAX: usize = 255;
    if name.len() > NAME_MAX {
        return Err(SysError::ENAMETOOLONG);
    }
    let current_uid = task.euid();
    log::info!("unlink paramter name: {}", name);
    let mut registry = MQ_REGISTRY.lock();
    if let Some(queue) = registry.queues.get(&name) {
        if current_uid != 0 && queue.owner_uid != current_uid {
            return Err(SysError::EACCES);
        }else {
            registry.queues.remove(&name);
            return Ok(0);
        }
    } else {
        Err(SysError::ENOENT)
    }
}

// Helper to map MqError to SysError
fn map_mq_error(e: MqError) -> SysError {
    match e {
        MqError::TimedOut => SysError::ETIMEOUT,
        MqError::MsgTooBig => SysError::EMSGSIZE,
        MqError::WouldBlock => SysError::EAGAIN,
        MqError::InvalidHandle => SysError::EBADF,
        MqError::PermissionDenied => SysError::EACCES,
    }
}

// --- 1. mq_getsetattr ---
// POSIX has mq_getattr and mq_setattr. We can combine them.
// Note: POSIX mq_setattr can only change mq_flags. Other changes are ignored.
pub fn sys_mq_getsetattr(handle: usize, new_attr_ptr: usize, old_attr_ptr: usize) -> SysResult {
    let task = current_task().unwrap();
    let queue = task.with_fd_table(|table|{
        table.get_file(handle)})?
    .downcast_arc::<MessageQueue>().map_err(|_| SysError::EINVAL)?;
    let mut vm = task.get_vm_space().lock(); 

    // 如果 old_attr_ptr 非空, 获取当前属性并写入用户空间。
    if old_attr_ptr != 0 {
        let old_attr_user_ptr = UserPtrRaw::new(old_attr_ptr as *mut MqAttr)
            .ensure_write(&mut vm) // 修正了笔误 ensure_writee
            .ok_or(SysError::EFAULT)?;
        
        let current_attr = queue.inner.lock().attr;

        old_attr_user_ptr.write(current_attr);
    }

    // 如果 new_attr_ptr 非空, 从用户空间读取新属性并设置。
    if new_attr_ptr != 0 {
        let new_attr_user_ptr = UserPtrRaw::new(new_attr_ptr as *const MqAttr)
            .ensure_read(&mut vm)
            .ok_or(SysError::EFAULT)?;
        
        let new_attr = *new_attr_user_ptr.to_ref();
        
        let mut inner = queue.inner.lock();
        inner.attr.mq_flags = new_attr.mq_flags;
    }

    Ok(0)
}

pub fn sys_mq_notify(handle: usize, sevp_ptr: usize) -> SysResult {
    let task = current_task().unwrap();
    let queue = task.with_fd_table(|table|{
        table.get_file(handle)})?
    .downcast_arc::<MessageQueue>().map_err(|_| SysError::EINVAL)?;
    
    if sevp_ptr == 0 {
        queue.inner.lock().notify = None;
    } else {
        // 注册通知，需要读取 sigevent 结构体
        let mut vm = task.get_vm_space().lock();

        // 1. 创建读指针并校验 sigevent 结构
        let sevp_user_ptr = UserPtrRaw::new(sevp_ptr as *const Sigevent) // 使用 libc::sigevent
            .ensure_read(&mut vm)
            .ok_or(SysError::EFAULT)?;

        // 2. 安全地读取 sigevent 内容
        let event = *sevp_user_ptr.to_ref();
        
        match event.sigev_notify {
            SIGEV_SIGNAL | SIGEV_NONE => {}
            _ => {
                log::warn!("unsupported sigev_notify: {}", event.sigev_notify);
                return Err(SysError::EINVAL);
            }
        }

        // 3. 创建并存储注册信息
        let registration = NotifyRegistration {
            task: task.clone(),
            event,
        };

        let mut inner = queue.inner.lock();
        if inner.notify.is_some() {
            return Err(SysError::EBUSY);
        }
        inner.notify = Some(registration);
    }

    Ok(0)
}

fn abs_timespec_to_rel_timeout(ts: &TimeSpec) -> Option<Duration> {
    let abs: Duration = (*ts).into();
    let now = get_current_time_duration(); 

    if abs <= now {
        Some(Duration::from_secs(0))
    } else {
        Some(abs - now)
    }
}

const MSG_MAX_SIZE: usize = 8192;
pub async fn sys_mq_timedsend(handle: usize, msg_ptr: usize, msg_len: usize, msg_prio: usize, timeout_ptr: usize) -> SysResult {
    log::warn!("handle: {}, msg_ptr: {:#x}, msg_len: {}, msg_prio: {}, timeout_ptr: {:#x}", handle, msg_ptr, msg_len, msg_prio, timeout_ptr);
    let task = current_task().unwrap();
    let queue = task.with_fd_table(|table|{
        table.get_file(handle)})?
    .downcast_arc::<MessageQueue>().map_err(|_| SysError::EBADF)?;
    
    let attr_len = queue.inner.lock().attr.mq_msgsize;
    if msg_len >  MSG_MAX_SIZE{
        log::warn!("[sys_mq_timedsend] msg_len: {}, queue.attr.mq_msgsize: {}", msg_len, attr_len);
        return Err(SysError::EMSGSIZE);
    }
    
    let msg_user_slice = UserSliceRaw::new(msg_ptr as *const u8, msg_len)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;

    let msg_data = msg_user_slice.to_ref();

    // 2. 确定超时时间
    let timeout = if timeout_ptr == 0 {
        None
    } else {
        let timeout_user_ptr = UserPtrRaw::new(timeout_ptr as *const TimeSpec)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        let timespec = timeout_user_ptr.to_ref();
        if !timespec.is_valid() {
            log::warn!("[sys_mq_timedsend] invalid timeout: {:?}", timespec);
            return Err(SysError::EINVAL);
        }
        match abs_timespec_to_rel_timeout(&timespec) {
            Some(d) => Some(d),
            None    => Some(Duration::from_secs(0)), // 立即超时
        }
    };
    log::warn!("timeout: {:?}", timeout);
    // 3. 阻塞并执行异步操作 (这部分逻辑不变)
    // let future = queue.mq_timedsend(&msg_data, msg_prio, timeout);
    // match block_on_mq(future) {
    //     Ok(()) => Ok(0),
    //     Err(e) => Err(map_mq_error(e)),
    // }
    block_on_mq(
        queue.mq_timedsend(&msg_data, msg_prio as u32, timeout)
    ).await?;
    Ok(0)

}

// --- 4. sys_mq_timedreceive / sys_mq_receive (重构) ---
pub async fn sys_mq_timedreceive(handle: usize, msg_ptr: usize, msg_len: usize, msg_prio_ptr: usize, timeout_ptr: usize) -> SysResult {
    log::warn!("handle: {}, msg_ptr: {:#x}, msg_len: {}, msg_prio: {}, timeout_ptr: {:#x}", handle, msg_ptr, msg_len, msg_prio_ptr, timeout_ptr);
    let task = current_task().unwrap();
    let queue = task.with_fd_table(|table|{
        table.get_file(handle)})?
    .downcast_arc::<MessageQueue>().map_err(|_| SysError::EBADF)?;
    
    // 检查用户缓冲区大小
    let attr_len = queue.inner.lock().attr.mq_msgsize;
    if msg_len >  MSG_MAX_SIZE {
        log::warn!("[sys_mq_timedreceive] msg_len: {}, queue.attr.mq_msgsize: {}", msg_len, attr_len);
        return Err(SysError::EMSGSIZE);
    }

    // 1. 确定超时时间 (逻辑与 send 相同)
    let timeout = if timeout_ptr == 0 {
        None
    } else {
        let timeout_user_ptr = UserPtrRaw::new(timeout_ptr as *const TimeSpec)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        let timespec = timeout_user_ptr.to_ref();
        if !timespec.is_valid() {
            log::warn!("[sys_mq_timedsend] invalid timeout: {:?}", timespec);
            return Err(SysError::EINVAL);
        }
        match abs_timespec_to_rel_timeout(&timespec) {
            Some(d) => Some(d),
            None    => Some(Duration::from_secs(0)), // 立即超时
        }
    };
    log::warn!("timeout: {:?}", timeout);
    let (data, priority) = block_on_mq(
        queue.mq_timedreceive(timeout)
    ).await?;
    let msg_buf_user_slice = UserSliceRaw::new(msg_ptr as *mut u8, data.len())
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
    msg_buf_user_slice.to_mut().copy_from_slice(&data);

    // 3.2 如果需要，写入消息优先级
    if msg_prio_ptr != 0 {
        let prio_user_ptr = UserPtrRaw::new(msg_prio_ptr as *mut u32)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        prio_user_ptr.write(priority);
    }

    // 成功时返回消息的长度
    Ok(data.len() as isize)

}


async fn block_on_mq<T, F>(fut: F ) -> Result<T, SysError>
where
    F: core::future::Future<Output = Result<T, MqError>> + Send ,    
{
    let mut fut = Box::pin(fut);// 固定 future 的位置
    loop {
        match fut.as_mut().await {
            Ok(res) => return Ok(res),
            Err(MqError::WouldBlock) => {
                log::warn!("[MQ_block_on] EAGAIN");
                suspend_now().await;

                let task = current_task().unwrap();
                let has_signal_flag = task.with_sig_manager(|sig_manager| {
                    let block_sig = sig_manager.blocked_sigs;
                    sig_manager.check_pending_flag(!block_sig | SigSet::SIGKILL | SigSet::SIGALRM)
                });

                if has_signal_flag {
                    log::warn!("[block_on] has signal flag, return EINTR");
                    return Err(SysError::EINTR);
                }
            }
            Err(e) => return Err(map_mq_error(e)),
        }
    }
}

pub async fn sys_epoll_pwait2(epfd: usize, events_ptr: usize, maxenvets: usize, timeout_ptr: usize, sigmask_ptr: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let ep_inst_file = task.with_fd_table(|t| t.get_file(epfd))?;
    let ep_inst = match ep_inst_file.downcast_arc::<EPollInstance>() {
        Ok(inst) => inst,
        _ => return Err(SysError::EINVAL)
    };
    if (maxenvets as isize) <= 0 {
        return Err(SysError::EINVAL)
    }
    let events = UserSliceRaw::new(events_ptr as *mut EPollEvent, maxenvets)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;

    // check the ready list, if not empty, return immediately
    match ep_inst.clone().get_ready(events.to_mut()) {
        0 => {},
        nfds => return Ok(nfds as isize)
    }
     
    // no ready events, start to wait
    let timeout = match timeout_ptr {
        0 => None, // return immediately, even no ready event
        _ => {
            let ts_ptr = UserPtrRaw::new(timeout_ptr as *const TimeSpec)
                .ensure_read(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
            Some(*ts_ptr.to_ref())
        }
    };

    let old_sigmask = task.sig_manager.lock().get_sigmask();
    let mut new_sigmask = old_sigmask;
    if sigmask_ptr != 0 {
        let sigmask_ptr = UserPtrRaw::new(sigmask_ptr as *const SigSet)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
        new_sigmask = *sigmask_ptr.to_ref();
    }
    task.sig_manager.lock().set_sigmask(new_sigmask);
    
    let intr_future = IntrBySignalFuture { task: task.clone(), mask: new_sigmask };
    let epoll_future = EPollFuture { epoll_inst: ep_inst.clone() };

    task.set_interruptable();
    task.set_wake_up_sigs(!new_sigmask);

    if let Some(timeout) = timeout {
        // select from intr and timedtask and epoll event
        let timed_future = TimedTaskFuture::new(
            timeout.into(), 
            epoll_future
        );
        let sel_res = Select2Futures::new(timed_future, intr_future).await;
        // wake up, should restore states before return to user
        task.set_running();
        task.sig_manager.lock().set_sigmask(old_sigmask);
        match sel_res {
            SelectOutput::Output1(_) => {
                let nfds = ep_inst.get_ready(events.to_mut());
                return Ok(nfds as isize)
            }
            SelectOutput::Output2(_) => return Err(SysError::EINTR)
        }
    } else {
        // select from intr and epoll event
        let sel_res = Select2Futures::new(epoll_future, intr_future).await;
        task.set_running();
        task.sig_manager.lock().set_sigmask(old_sigmask);
        match sel_res {
            SelectOutput::Output1(_) => {
                let nfds = ep_inst.get_ready(events.to_mut());
                return Ok(nfds as isize)
            }
            SelectOutput::Output2(_) => return Err(SysError::EINTR)
        }
    }
}
