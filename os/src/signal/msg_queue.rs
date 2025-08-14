use alloc::boxed::Box;
use alloc::collections::{BinaryHeap, VecDeque};
use alloc::sync::Arc;
use alloc::vec::Vec;
use async_trait::async_trait;
use spin::Mutex;
use core::future::Future;
use core::pin::{pin, Pin};
use core::sync::atomic::AtomicUsize;
use core::task::{Context, Poll, Waker};
use core::time::Duration;

use crate::fs::vfs::{File, FileInner};
use crate::signal::SigInfo;
use crate::sync::mutex::SpinNoIrqLock; use crate::syscall::SysError;
use crate::task::current_task;
use crate::task::task::TaskControlBlock;
// Using your provided lock
use crate::timer::timed_task::{TimedTaskFuture, TimedTaskOutput}; // Using your provided async timer

/// Represents a message in the queue.
/// We derive Ord, PartialOrd, etc., to make it work in a BinaryHeap.
/// Note: BinaryHeap is a max-heap, which aligns with POSIX priorities (higher number is higher priority).
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Message {
    priority: u32,
    data: Vec<u8>,
}

// Implement Ord manually to control the comparison based on priority
impl Ord for Message {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// POSIX Message Queue attributes.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MqAttr {
    pub mq_flags: i64,   // Flags (O_NONBLOCK, etc.) 
    pub mq_maxmsg: usize, // Max number of messages in queue.
    pub mq_msgsize: usize, // Max message size in bytes.
}

impl MqAttr {
    pub fn is_valid(&self) -> bool {
        self.mq_flags >= 0 && self.mq_maxmsg > 0 && self.mq_msgsize > 0 && self.mq_maxmsg < 65536 && self.mq_msgsize < 8192
    }
}

/// Error types for message queue operations.
#[derive(Debug, PartialEq, Eq)]
pub enum MqError {
    TimedOut,
    MsgTooBig,
    PermissionDenied, // Placeholder for future extensions
    InvalidHandle,    // Placeholder
    WouldBlock,       // Placeholder for non-blocking operations
}

/// The internal state of a message queue, protected by a lock.
pub struct MessageQueueInner {
    pub attr: MqAttr,
    pub messages: BinaryHeap<Message>,
    sender_wakers: Option<Waker>,
    receiver_wakers: Option<Waker>,
    pub notify: Option<NotifyRegistration>, 
}

unsafe impl  Send for MessageQueueInner {}
unsafe impl  Sync for MessageQueueInner {}
pub struct NotifyRegistration {
    pub task: Arc<TaskControlBlock>,
    pub event: Sigevent,
}

/// A handle to a POSIX-style message queue.
/// It uses Arc to allow multiple handles to the same queue.
#[derive(Clone)]
pub struct MessageQueue {
    pub inner: Arc<SpinNoIrqLock<MessageQueueInner>>,
}

unsafe impl Send for MessageQueue {}
unsafe impl Sync for MessageQueue {}
// In the same file `ipc/mqueue.rs`

impl MessageQueue {
    /// Creates a new message queue with the given attributes.
    pub fn new(attr: MqAttr) -> Self {
        log::warn!("[mq_open] attr: mq_flags={}, mq_maxmsg={}, mq_msgsize={}",
           attr.mq_flags, attr.mq_maxmsg, attr.mq_msgsize);
        Self {
            inner: Arc::new(SpinNoIrqLock::new(MessageQueueInner {
                attr,
                messages: BinaryHeap::with_capacity(attr.mq_maxmsg),
                sender_wakers: None,
                receiver_wakers: None,
                notify: None,
            })),
        }
    }

    /// Asynchronously sends a message to the queue, waiting up to `timeout` if the queue is full.
    pub async fn mq_timedsend(
        &self,
        data: &[u8],
        priority: u32,
        timeout: Option<Duration>,
    ) -> Result<isize, MqError> {
        // Wrap the core logic (SendFuture) with your TimedTaskFuture.
        // This beautifully handles the timeout.
        let send_future = SendFuture {
            queue: self.clone(),
            message_data: data.to_vec(),
            message_priority: priority,
        };
        match timeout {
            Some(d) => {
                match TimedTaskFuture::new(d, send_future).await {
                    TimedTaskOutput::OK(result) => match result{
                        Ok(()) => Ok(0),
                        Err(e) => Err(e),
                    },
                    TimedTaskOutput::TimedOut => Err(MqError::TimedOut),
                }
            },
            None => {
                send_future.await?;
                Ok(0)
            }
        }
        
    }

    /// Asynchronously receives a message, waiting up to `timeout` if the queue is empty.
    pub async fn mq_timedreceive(
        &self,
        timeout: Option<Duration>,
    ) -> Result<(Vec<u8>, u32), MqError> {
        // Symmetrically, wrap the ReceiveFuture with TimedTaskFuture.
        let receive_future = ReceiveFuture { queue: self.clone() };

        match timeout {
            Some(d) => {
                match TimedTaskFuture::new(d, receive_future).await {
                    TimedTaskOutput::OK(result) => result,
                    TimedTaskOutput::TimedOut => Err(MqError::TimedOut),
                }
            },
            None => {
                receive_future.await
            }
        }
    }
}

#[async_trait]
impl File for MessageQueue {
    #[doc = " get basic File object"]
    fn file_inner(&self) ->  &FileInner {
        todo!()
    }

    #[doc = " If readable"]
    fn readable(&self) -> bool {
        false
    }

    #[doc = " If writable"]
    fn writable(&self) -> bool {
        false
    }

    #[doc = " Read file, will adjust file offset"]
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    async fn read(&self,_buf: &mut [u8]) -> Result<usize,SysError> {
        Err(SysError::EINVAL)
    }

    #[doc = " Write file, will adjust file offset"]
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    async fn write(&self,_buf: &[u8]) -> Result<usize,SysError> {
        Err(SysError::EINVAL)
    }
}
// In the same file `ipc/mqueue.rs`

// --- Private Future for Sending ---
struct SendFuture {
    queue: MessageQueue,
    message_data: Vec<u8>,
    message_priority: u32,
}

pub const MQ_FLAG_NONBLOCK: i64 = 0x800;
impl Future for SendFuture {
    type Output = Result<(), MqError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.queue.inner.lock();

        // // 1. Check if message is too large
        if self.message_data.len() > inner.attr.mq_msgsize {
            return Poll::Ready(Err(MqError::MsgTooBig));
        }

        // 2. Check if the queue is full
        if inner.messages.len() >= inner.attr.mq_maxmsg {
            // Queue is full, we need to wait.
            // Store the waker so we can be woken up later.
            if inner.attr.mq_flags & MQ_FLAG_NONBLOCK as i64 != 0{
                return Poll::Ready(Err(MqError::WouldBlock));
            }
           if inner.sender_wakers.as_ref().map_or(true, |w| !w.will_wake(cx.waker())){
                inner.sender_wakers = Some(cx.waker().clone());
           }
            Poll::Pending
        } else {
            let was_empty = inner.messages.is_empty();
            // Queue has space, proceed to send.
            let message = Message {
                priority: self.message_priority,
                data: self.message_data.clone(), // This clone can be optimized if needed
            };
            inner.messages.push(message);

            // After sending, check if there's a receiver waiting.
            // If so, wake them up!
            if let Some(waker) = inner.receiver_wakers.take() {
                waker.wake();
            }else if was_empty {
                if let Some(registration) = inner.notify.take() {
                    let sender_task = current_task().unwrap();
                    let sender_pid = sender_task.pid();
                    dispatch_notification(&registration, sender_pid);
                }
            }

            Poll::Ready(Ok(()))
        }
    }
}
unsafe impl Send for SendFuture {}
unsafe impl Sync for SendFuture {}

// --- Private Future for Receiving ---
struct ReceiveFuture {
    queue: MessageQueue,
}

impl Future for ReceiveFuture {
    type Output = Result<(Vec<u8>, u32), MqError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.queue.inner.lock();

        // 1. Check if the queue is empty
        if let Some(message) = inner.messages.pop() {
            // Queue has a message, proceed to receive.
            // After receiving, there is now space in the queue.
            // Check if there's a sender waiting for space.
            if let Some(waker) = inner.sender_wakers.take() {
                waker.wake();
            }
            Poll::Ready(Ok((message.data, message.priority)))
        } else {
            if inner.attr.mq_flags & MQ_FLAG_NONBLOCK as i64 != 0{
                return Poll::Ready(Err(MqError::WouldBlock));
            }
            // Queue is empty, we need to wait.
            // Store the waker so we can be woken up when a message arrives.
            if inner.receiver_wakers.as_ref().map_or(true, |w| !w.will_wake(cx.waker())){
                inner.receiver_wakers = Some(cx.waker().clone());
            }        
            Poll::Pending
        }
    }
}

unsafe impl Send for ReceiveFuture {}
unsafe impl Sync for ReceiveFuture {}

// --- POSIX 常量 ---
pub const SIGEV_SIGNAL: i32 = 0; // send signo  
pub const SIGEV_NONE: i32 = 1;   // do nothing  
pub const SIGEV_THREAD: i32 = 2;   // create a kernel thread to run function , not implemented yet 

#[repr(C)]
#[derive(Copy, Clone)]
pub union Sigval {
    pub sival_int: i32,
    pub sival_ptr: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Sigevent {
    pub sigev_notify: i32,
    pub sigev_signo: i32,
    pub sigev_value: Sigval, // todo: kernel now ignore this field
    // 以下字段因不支持 SIGEV_THREAD 而被忽略
    pub sigev_notify_function: usize,
    pub sigev_notify_attributes: usize,
}

pub fn dispatch_notification(registration: &NotifyRegistration, sender_pid: usize) {
    match registration.event.sigev_notify {
        SIGEV_SIGNAL => {
            let sig_info = SigInfo {
                si_signo: registration.event.sigev_signo as usize,
                si_code: SigInfo::MESGQ,
                si_pid: Some(sender_pid),
            };
            registration.task.recv_sigs_process_level(sig_info);
        }
        SIGEV_NONE => {
            // 不做任何事
        }
        _ => {
            // 对于不支持的类型，例如 SIGEV_THREAD，仅在内核中打印日志。
            // 真正的错误检查应该在 sys_mq_notify 中完成。
            log::warn!(
                "dispatch_notification: Unsupported sigev_notify type {}",
                registration.event.sigev_notify
            );
        }
    }
}