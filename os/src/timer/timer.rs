use core::{
    cmp::Reverse,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};
extern crate alloc;
use alloc::{
    boxed::Box,
    collections::BinaryHeap,
    sync::{Arc, Weak},
    vec::Vec,
};
use async_trait::async_trait;
use downcast_rs::DowncastSync;
use log::info;

use super::{ffi::TimeVal, get_current_time_duration};
use crate::{
    devices::net::NetRxToken,
    fs::vfs::{File, FileInner},
    processor::processor::current_processor,
    signal::{
        msg_queue::{Sigevent, SIGEV_SIGNAL},
        SigInfo, SIGALRM,
    },
    sync::mutex::SpinNoIrqLock,
    syscall::{SysError, SysResult},
    task::task::TaskControlBlock,
    timer::{
        ffi::TimeSpec,
        timed_task::{PendingFuture, TimedTaskFuture},
    },
};
use hal::{
    board::MAX_PROCESSORS,
    instruction::{Instruction, InstructionHal},
};
use spin::{Lazy, Mutex};
/// A trait that defines the event to be triggered when a timer expires.
/// The TimerEvent trait requires a callback method to be implemented,
/// which will be called when the timer expires.
pub trait TimerEvent: Send + Sync {
    /// The callback method to be called when the timer expires.
    /// This method consumes the event data and optionally returns a new timer.
    ///
    /// # Returns
    /// An optional Timer object that can be used to schedule another timer.
    fn callback(self: Box<Self>) -> Option<Timer>;
}

/// Represents a timer with an expiration time and associated event data.
/// The Timer structure contains the expiration time and the data required
/// to handle the event when the timer expires.
pub struct Timer {
    /// The expiration time of the timer.
    /// This indicates when the timer is set to trigger.
    pub expire: Duration,

    /// A boxed dynamic trait object that implements the TimerEvent trait.
    /// This allows different types of events to be associated with the timer.
    pub data: Box<dyn TimerEvent>,
}

impl Timer {
    /// new a Timer
    pub fn new(expire: Duration, data: Box<dyn TimerEvent>) -> Self {
        Self { expire, data }
    }
    /// new a Timer for Waker event
    pub fn new_waker_timer(expire: Duration, waker: Waker) -> Self {
        struct WakerData {
            waker: Waker,
        }
        impl TimerEvent for WakerData {
            fn callback(self: Box<Self>) -> Option<Timer> {
                self.waker.wake();
                None
            }
        }
        Self {
            expire,
            data: Box::new(WakerData { waker }),
        }
    }

    fn callback(self) -> Option<Timer> {
        self.data.callback()
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.expire.cmp(&other.expire)
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Timer {}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.expire == other.expire
    }
}

/// `TimerManager` is responsible for managing all the timers in the system.
/// It uses a thread-safe lock to protect a priority queue (binary heap) that
/// stores the timers. The timers are stored in a `BinaryHeap` with their
/// expiration times wrapped in `Reverse` to create a min-heap, ensuring that
/// the timer with the earliest expiration time is at the top.
pub struct TimerManager {
    /// A priority queue to store the timers. The queue is protected by a spin
    /// lock to ensure thread-safe access. The timers are wrapped in
    /// `Reverse` to maintain a min-heap.
    timers: SpinNoIrqLock<BinaryHeap<Reverse<Timer>>>,
}

impl TimerManager {
    fn new() -> Self {
        Self {
            timers: SpinNoIrqLock::new(BinaryHeap::new()),
        }
    }
    /// add a timer for Manager
    pub fn add_timer(&self, timer: Timer) {
        log::debug!("add new timer, next expiration {:?}", timer.expire);
        self.timers.lock().push(Reverse(timer));
    }
    /// check for the manager
    pub fn check(&self) {
        loop {
            let mut timers = self.timers.lock();
            if let Some(timer) = timers.peek() {
                let current_time = get_current_time_duration();
                if current_time >= timer.0.expire {
                    log::trace!("timers len {}", timers.len());

                    // info!(
                    //     "[Timer Manager] there is a timer expired, current:{:?}, expire:{:?}",
                    //     current_time,
                    //     timer.0.expire
                    // );

                    let timer = timers.pop().unwrap().0;
                    drop(timers);
                    if let Some(new_timer) = timer.callback() {
                        self.timers.lock().push(Reverse(new_timer));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}
/// The global `TimerManager` instance that can be accessed from anywhere in the kernel.
pub static TIMER_MANAGER: Lazy<TimerManager> = Lazy::new(TimerManager::new);

/// below are timer structure in linux,ITimer is a timer struct in linux used in settimmer
///and in get timer, ther are three types of timer in linux
#[derive(Debug)]
pub struct ITimer {
    /// interval: repeat gap
    pub interval: Duration,
    /// next_expire_time
    pub next_expire: Duration,
    /// timer id
    pub id: usize,
}

impl ITimer {
    /// zero itimer
    pub const ZERO: Self = Self {
        interval: Duration::ZERO,
        next_expire: Duration::ZERO,
        id: 0,
    };
}

static TIMER_ID_ALLOCATOR: AtomicUsize = AtomicUsize::new(1);

/// alloc func for itimer id
pub fn alloc_timer_id() -> usize {
    TIMER_ID_ALLOCATOR.fetch_add(1, Ordering::Relaxed)
}

/// a timer mechanism called itimer for implementing interval timers.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct ITimerVal {
    /// Interval for periodic timer
    pub it_interval: TimeVal,
    /// Time until next expiration
    pub it_value: TimeVal,
}

impl ITimerVal {
    /// a zero ITimerVal const
    pub const ZERO: Self = Self {
        it_interval: TimeVal::ZERO,
        it_value: TimeVal::ZERO,
    };
    /// check the val is valid
    pub fn is_valid(&self) -> bool {
        self.it_interval.usec < 1_000_000 && self.it_value.usec < 1_000_000
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct ITimerSpec {
    pub it_interval: TimeSpec,
    pub it_value: TimeSpec,
}

impl ITimerSpec {
    pub fn is_valid(&self) -> bool {
        self.it_interval.is_valid() && self.it_value.is_valid()
    }
}

#[derive(Default, Debug)]
/// based on real time no matter the task is running the timer will work
/// poll by SIGALRM
pub struct RealITimer {
    /// tcb in RealITimer
    pub task: Weak<TaskControlBlock>,
    /// id of the timer
    pub id: usize,
}

impl TimerEvent for RealITimer {
    fn callback(self: Box<Self>) -> Option<Timer> {
        self.task.upgrade().and_then(|task| {
            task.with_mut_itimers(|itimers| {
                let real_timer = &mut itimers[0];
                if real_timer.id != self.id {
                    log::warn!("check failed!");
                    return None;
                }
                task.recv_sigs_process_level(SigInfo {
                    si_signo: SIGALRM,
                    si_code: SigInfo::KERNEL,
                    si_pid: None,
                });
                let real_timer_interval = real_timer.interval;
                if real_timer_interval == Duration::ZERO {
                    return None;
                }
                let next_expire = get_current_time_duration() + real_timer_interval;
                real_timer.next_expire = next_expire;
                Some(Timer {
                    expire: next_expire,
                    data: self,
                })
            })
        })
    }
}

pub type TimerId = u32;

pub struct PosixTimer {
    /// tcb in PosixTimer
    pub task: Weak<TaskControlBlock>,
    pub sigevent: Sigevent,
    pub interval: Duration,
    pub next_expire: Duration,
    /// check if has been replace
    pub interval_id: TimerId,
    /// last 'sent' signo orverun count
    pub last_overrun: usize,
}

impl TimerEvent for PosixTimer {
    fn callback(mut self: Box<Self>) -> Option<Timer> {
        let task = match self.task.upgrade() {
            Some(t) => t,
            None => return None,
        };

        let mut posix_timers = task.posix_timers.lock();

        let timer_id = unsafe { self.sigevent.sigev_value.sival_ptr } as TimerId;

        // 定时器可能已被删除
        let timer_entry = match posix_timers.get_mut(&timer_id) {
            Some(t) => t,
            None => return None,
        };

        // 版本号不一致：旧回调，直接失效
        if timer_entry.interval_id != self.interval_id {
            return None;
        }

        let now = get_current_time_duration();
        let mut overrun = 0usize;

        // 仅当当前确实到期（或已过期）才投递信号
        if now >= self.next_expire {
            // 计算错过了多少个周期（k-1）
            if self.interval > Duration::ZERO {
                let late = now.saturating_sub(self.next_expire);
                // k = floor(late/interval) + 1   （至少为 1）
                let k = (late.as_nanos() / self.interval.as_nanos() as u128) as usize + 1;
                overrun = k.saturating_sub(1);

                // 刷新下一次到期（跳过多个周期）
                self.next_expire = self.next_expire + self.interval * (k as u32);
            } else {
                // one-shot：下一次到期清零（不再重启）
                self.next_expire = Duration::ZERO;
            }

            // 记录 overrun 到 map，符合 “最近一次已投递信号的 overrun”
            timer_entry.last_overrun = overrun;

            // 同步 map 中的 next_expire
            timer_entry.next_expire = self.next_expire;

            // 发送信号（仅支持 SIGEV_SIGNAL）
            match self.sigevent.sigev_notify {
                SIGEV_SIGNAL => {
                    let sig_info = SigInfo {
                        si_signo: self.sigevent.sigev_signo as usize,
                        si_code: SigInfo::KERNEL,
                        si_pid: None,
                    };
                    task.recv_sigs_process_level(sig_info);
                }
                _ => {
                    log::warn!(
                        "Unsupported sigev_notify value: {}",
                        self.sigevent.sigev_notify
                    );
                }
            }

            // 若为周期定时器，则把“下一次”重新入队；否则结束
            if self.interval > Duration::ZERO && self.next_expire > Duration::ZERO {
                // 递归沿用同一 interval_id，直到下一次 settime/disarm 才会递增
                let next = Timer {
                    expire: self.next_expire,
                    data: self,
                };
                return Some(next);
            } else {
                return None;
            }
        }

        // 未到期则不应被调用（正常不会进入这里；守稳返回 None）。
        None
    }
}

pub struct TimerFdFile {
    pub file_inner: FileInner,
    pub timer: Arc<Mutex<TimerFd>>,
}

unsafe impl Send for TimerFdFile {}
unsafe impl Sync for TimerFdFile {}

#[derive(Clone)]
pub struct TimerFd {
    pub task: Weak<TaskControlBlock>,
    pub interval: Duration,
    pub next_expire: Duration,
    pub interval_id: TimerId,
    pub expirations: u64,
    // 当 poll 被调用时，waker 会被设置。
    pub wait_future: Option<TimerFdReadFuture>,
}

#[async_trait]
impl File for TimerFdFile {
    fn file_inner(&self) -> &FileInner {
        &self.file_inner
    }

    fn readable(&self) -> bool {
        self.timer.lock().expirations > 0
    }

    fn writable(&self) -> bool {
        false
    }

    async fn read(&self, buf: &mut [u8]) -> Result<usize, SysError> {
        let expirations;
        {
            let mut inner = self.timer.lock();
            if inner.expirations == 0 {
                let fut = inner
                    .wait_future
                    .get_or_insert_with(|| TimerFdReadFuture::new(Arc::clone(&self.timer)))
                    .clone();
                drop(inner); 
                fut.await;
                inner = self.timer.lock(); 
            }
            expirations = inner.expirations;
            inner.expirations = 0;
        } 
        if buf.len() < 8 {
            return Err(SysError::EINVAL);
        }
        buf[..8].copy_from_slice(&expirations.to_ne_bytes());
        Ok(8)
    }
    async fn write(&self, _buf: &[u8]) -> Result<usize, SysError> {
        Err(SysError::EINVAL)
    }
}

#[derive(Clone)]
pub struct TimerFdReadFuture {
    timer: Arc<Mutex<TimerFd>>,
    waker: Option<Waker>,
}

impl TimerFdReadFuture {
    pub fn new(timer: Arc<Mutex<TimerFd>>) -> Self {
        Self { timer, waker: None }
    }

    /// Called by timer callback to wake up the future
    pub fn wake(&self) {
        if let Some(w) = &self.waker {
            w.wake_by_ref();
        }
    }
}

impl Future for TimerFdReadFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let inner = self.timer.lock();

        // 检查是否有到期事件
        if inner.expirations > 0 {
            // 如果有，就绪
            Poll::Ready(())
        } else {
            // 没有到期事件，并且定时器未设置，则立即就绪
            // 这对应于 timerfd_settime 设置 it_value 为 0 的情况。
            if inner.next_expire.is_zero() {
                Poll::Ready(())
            } else {
                // 定时器已设置但未到期，我们需要等待
                // 更新 Waker
                if self.waker.as_ref().map(|w| !w.will_wake(cx.waker())).unwrap_or(true) {
                    drop(inner);
                    self.waker = Some(cx.waker().clone());
                }
                // 将 Waker 注册到全局定时器管理器
                // 确保在到期时会唤醒当前任务
                // 避免每次 poll 都添加新的定时器。
                TIMER_MANAGER.add_timer(Timer::new_waker_timer(
                    self.timer.lock().next_expire,
                    cx.waker().clone(),
                ));

                Poll::Pending
            }
        }
    }
}

pub struct TimerFdEvent(pub Arc<Mutex<TimerFd>>);
impl TimerEvent for TimerFdEvent {
    fn callback(self: Box<Self>) -> Option<Timer> {
        let mut inner = self.0.lock();
        inner.expirations += 1;
        if let Some(fut) = &inner.wait_future {
            fut.wake();
        }
        if inner.interval > Duration::ZERO {
            let next_expire = get_current_time_duration() + inner.interval;
            inner.next_expire = next_expire;
            drop(inner); // 释放锁
            Some(Timer::new(next_expire, self)) // 再次将自己放回 TIMER_MANAGER
        } else {
            inner.next_expire = Duration::ZERO;
            None
        }
    }
}