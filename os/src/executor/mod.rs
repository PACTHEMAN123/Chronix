use alloc::collections::VecDeque;
use lazy_static::*;
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use log::info;
use core::future::Future;
use crate::sync::mutex::SpinNoIrqLock;

struct TaskQueue {
    queue: SpinNoIrqLock<VecDeque<Runnable>>,
}
#[allow(dead_code)]
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

    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.lock().len() as usize
    }
}

static TASK_QUEUE: TaskQueue = TaskQueue::new();

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
        TASK_QUEUE.push(runnable);
        info!("spawn a runnable now");
    };
    async_task::spawn(future, WithInfo(schedule))
}

pub fn run_until_idle() -> usize{
    info!("now run in loop");
    let mut count = 0;
    while let Some(runnable) = TASK_QUEUE.fetch() {
        info!("already fetch a runnable");
        runnable.run();
        count += 1;
    }
    count
}

#[allow(unused)]
pub fn run_forever() {
    loop {
        if let Some(runnable) = TASK_QUEUE.fetch() {
            runnable.run();
        } 
    }
}
