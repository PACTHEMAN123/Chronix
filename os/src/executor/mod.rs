use alloc::collections::VecDeque;
use lazy_static::*;
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use log::info;
use core::future::Future;
use crate::processor;
use crate::sync::mutex::SpinNoIrqLock;
use crate::processor::processor::{current_processor, PROCESSORS};
#[cfg(not(feature = "smp"))]
pub struct TaskQueue {
    queue: SpinNoIrqLock<VecDeque<Runnable>>,
}
#[allow(dead_code)]
#[cfg(not(feature = "smp"))]
impl TaskQueue {
    pub const fn new() -> Self {
        Self {
            queue: SpinNoIrqLock::new(VecDeque::new()),
        }
    }
    
    pub fn init(&self)  {
        *self.queue.lock() = VecDeque::new();
    }
    pub fn push(&self, runnable: Runnable) {
        self.queue.lock().push_back(runnable);
    }

    pub fn fetch(&self) -> Option<Runnable> {
        self.queue.lock().pop_front()
    }   
    pub fn pop_back(&self) -> Option<Runnable> {
        self.queue.lock().pop_back()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.lock().len() as usize
    }
}
#[cfg(not(feature = "smp"))]
static TASK_QUEUE: TaskQueue = TaskQueue::new();
#[cfg(not(feature = "smp"))]
pub fn init() {
    TASK_QUEUE.init();
}
pub fn spawn<F>(future: F) -> (Runnable, Task<F::Output>)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
{
    let schedule= move |runnable:Runnable, _info: ScheduleInfo | {
        // todo: judge push method by ScheduleInfo
        #[cfg(not(feature = "smp"))]
        TASK_QUEUE.push(runnable);
        #[cfg(feature = "smp")]
        unsafe{PROCESSORS[crate::processor::schedule::select_run_queue_index()].unwrap_with_mut_task_queue(|task_queue|task_queue.push_back(runnable))};
    };
    async_task::spawn(future, WithInfo(schedule))
}

pub fn kernel_spawn<F>(future: F) -> (Runnable, Task<F::Output>)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
{
    let schedule= move |runnable:Runnable, _info: ScheduleInfo | {
        // todo: judge push method by ScheduleInfo
        #[cfg(not(feature = "smp"))]
        TASK_QUEUE.push(runnable);
        #[cfg(feature = "smp")]
        current_processor().unwrap_with_mut_task_queue(|task_queue|task_queue.push_back(runnable));
    };
    async_task::spawn(future, WithInfo(schedule))
}

pub fn run_until_idle() -> usize{
    let mut len = 0;
    #[cfg(not(feature = "smp"))]
    while let Some(runnable) = TASK_QUEUE.fetch() {
        //info!("already fetch a runnable");
        runnable.run();
        len += 1;
    }
    #[cfg(feature = "smp")]
    while let Some(runnable) = current_processor().unwrap_with_mut_task_queue(|task_queue| task_queue.pop_front()) {
        //info!("already fetch a runnable, runnable_num: {:?},current_processor_id: {}",current_processor().task_nums(),current_processor().id());
        runnable.run();
        len += 1;
    }
    len
}
