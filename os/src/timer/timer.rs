use core::{cmp::Reverse, sync::atomic::{AtomicUsize, Ordering}, task::Waker, time::Duration};
extern crate alloc;
use alloc::{boxed::Box, collections::BinaryHeap, sync::{Arc, Weak}};
use log::info;

use super::{ffi::TimeVal, get_current_time_duration};
use spin::Lazy;
use crate::{processor::processor::current_processor, signal::{SigInfo, SIGALRM}, sync::mutex::SpinNoIrqLock, task::task::TaskControlBlock};
use hal::{board::MAX_PROCESSORS, instruction::{Instruction, InstructionHal}};
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

#[derive (Default, Debug)]
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
        self.task.upgrade()
        .and_then(|task|{
              task.with_mut_itimers(|itimers| {
                    let real_timer = &mut itimers[0];
                    if real_timer.id != self.id {
                        log::warn!("check failed!");
                        return None
                    }
                    task.recv_sigs_process_level(
                        SigInfo { si_signo: SIGALRM, si_code: SigInfo::KERNEL, si_pid: None }
                    );
                    let real_timer_interval = real_timer.interval;
                    if real_timer_interval == Duration::ZERO {
                        return None;
                    }
                    let next_expire = get_current_time_duration() + real_timer_interval; 
                    real_timer.next_expire = next_expire;
                    Some(
                        Timer {
                            expire: next_expire,
                            data: self,
                        }
                    )
              })
        })
    }
}