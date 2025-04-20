use core::{
    clone, future::Future, pin::Pin, task::{Context, Poll}, time::Duration
};
use alloc::sync::Arc;
use log::info;

use crate::{task::task::TaskControlBlock, utils::suspend_now};

use super::{get_current_time_duration, timer::TIMER_MANAGER};
use super::timer::{Timer,TimerManager};

/// A future wrapper for a timed task.
pub struct TimedTaskFuture<F: Future + Send + 'static> {
    /// the specific time point when the task expires
    expire: Duration,
    /// the future which use the task
    future: F,
    /// whether the task is in the timer manager
    in_manager: bool,
}

impl <F: Future + Send + 'static> TimedTaskFuture<F> {
    /// Create a new timed task future.
    pub fn new(deadline: Duration, future: F) -> Self {
        Self {
            expire: get_current_time_duration() + deadline,
            future,
            in_manager: false,
        }
    }
}

/// The output of a timed task future.
pub enum TimedTaskOutput <T> {
    /// The task timed out.
    TimedOut,
    /// The task completed successfully.
    OK(T),
}
impl <F: Future + Send + 'static> Future for TimedTaskFuture<F> {
    type Output = TimedTaskOutput<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let ret = unsafe { Pin::new_unchecked(&mut this.future).poll(cx) };
        match ret {
            Poll::Pending => {
                if get_current_time_duration() >= this.expire {
                    log::info!("timed out");
                    Poll::Ready(TimedTaskOutput::TimedOut)
                }
                else {
                    if !this.in_manager {
                        TIMER_MANAGER.add_timer(Timer::new_waker_timer(this.expire, cx.waker().clone()));
                        this.in_manager = true;
                    }
                    Poll::Pending
                }
            }
            Poll::Ready(ret) => Poll::Ready(TimedTaskOutput::OK(ret)),
        }
    }
}

struct PendingFuture ;

impl Future for PendingFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Pending
    }
}
/// sleep for seconds
pub async fn ksleep(time: Duration) {
    TimedTaskFuture::new(time,PendingFuture{} ).await;
}
/// suspend out time out task future
pub async fn suspend_timeout(task: &Arc<TaskControlBlock>, time_limit: Duration) -> Duration {
    let expire = get_current_time_duration() + time_limit;
    TIMER_MANAGER.add_timer(Timer::new_waker_timer(expire, task.waker().clone().unwrap()));
    suspend_now().await;
    let now = get_current_time_duration();
    if expire > now {
        expire - now
    }
    else {
        Duration::ZERO
    }
}