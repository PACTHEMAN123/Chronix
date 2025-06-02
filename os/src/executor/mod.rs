use alloc::collections::VecDeque;
use alloc::task;
use hal::println;
use lazy_static::*;
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use log::info;
use alloc::sync::Arc;
use core::future::Future;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::processor;
use crate::signal::{SigInfo, SIGKILL, SIGTERM};
use crate::sync::mutex::SpinNoIrqLock;
use crate::processor::processor::{current_processor, PROCESSORS};
use crate::syscall::process;
use crate::task::manager::TASK_MANAGER;
use crate::task::INITPROC_PID;
use crate::task::{schedule::UserTaskFuture,task::TaskControlBlock};
use crate::timer::timed_task::suspend_timeout;
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
    pub fn push_preempt(&self, runnable: Runnable) {
        self.queue.lock().push_front(runnable);
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
pub fn spawn<F>(future: UserTaskFuture<F>) -> (Runnable, Task<F::Output>)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
{
    #[cfg(feature = "smp")]
    let cpu_mask_id = <Arc<TaskControlBlock> as Clone>::clone(&(&future.task.clone())).turn_cpu_mask_id();
    let schedule= move |runnable:Runnable, info: ScheduleInfo | {
            #[cfg(not(feature = "smp"))]
            if info.woken_while_running{
                TASK_QUEUE.push(runnable);
            }else {
                TASK_QUEUE.push_preempt(runnable);
            }
            #[cfg(feature = "smp")]
            if info.woken_while_running{
                unsafe{
                    if cpu_mask_id == 4 {
                        PROCESSORS[crate::processor::schedule::select_run_queue_index()]
                        .unwrap_with_mut_task_queue(|task_queue|task_queue.push_back(runnable))
                    } else {
                        PROCESSORS[cpu_mask_id]
                        .unwrap_with_mut_task_queue(|task_queue|task_queue.push_back(runnable))
                    }
                    
                };
            }else {
                unsafe{
                    if cpu_mask_id == 4 {
                        PROCESSORS[crate::processor::schedule::select_run_queue_index()]
                        .unwrap_with_mut_task_queue(|task_queue|task_queue.push_front(runnable))
                    } else {
                        PROCESSORS[cpu_mask_id]
                        .unwrap_with_mut_task_queue(|task_queue|task_queue.push_front(runnable))
                    }
                }
            }
            
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

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemStatus {
    Running = 0, 
    ShutingDown = 1,
    #[allow(unused)]
    Rebooting = 2,
}

impl From<usize> for SystemStatus {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Running,
            1 => Self::ShutingDown, 
            // 2 => Self::Rebooting,
            _ => panic!("system is in unexpected status")
        }
    }
}

static SYSTEM_STATUS: AtomicUsize = AtomicUsize::new(SystemStatus::Running as usize);

pub fn os_send_shutdown() {
    SYSTEM_STATUS.store(SystemStatus::ShutingDown as usize, Ordering::Release);
}

pub fn os_is_shutting_down() -> bool {
    SystemStatus::ShutingDown == SYSTEM_STATUS.load(Ordering::Acquire).into()
}

pub fn do_shutdown() -> Result<(), ()> {
    if TASK_MANAGER.has_task_except_initproc() {
        TASK_MANAGER.for_each_task(|task| {
            if task.tid() == INITPROC_PID || !task.is_leader() {
                return;
            }
            task.recv_sigs(SigInfo { si_signo: SIGKILL, si_code: SigInfo::KERNEL, si_pid: None });
        });
        Err(())
    } else {
        Ok(())
    }
}

pub fn run_until_idle() -> usize {
    let mut len = 0;
    #[cfg(not(feature = "smp"))]
    while let Some(runnable) = TASK_QUEUE.fetch() {
        //info!("already fetch a runnable");
        runnable.run();
        len += 1;
        if os_is_shutting_down() && do_shutdown().is_ok() {
            break;
        }
    }
    #[cfg(feature = "smp")]
    if current_processor().need_migrate_check() {
        let processor = current_processor();
        let migrate_id = processor.migrate_id();
        processor.set_need_migrate(processor.id());
        if let Some(migrate_runnable) = processor.unwrap_with_mut_task_queue(|task_queue| task_queue.pop_back()){
            unsafe{PROCESSORS[migrate_id].unwrap_with_mut_task_queue(|task_queue| task_queue.push_back(migrate_runnable))};
        }
    }
    #[cfg(feature = "smp")]
    while let Some(runnable) = current_processor().unwrap_with_mut_task_queue(|task_queue| task_queue.pop_front()) {
        //info!("already fetch a runnable, runnable_num: {:?},current_processor_id: {}",current_processor().task_nums(),current_processor().id());
        runnable.run();
        len += 1;
        if os_is_shutting_down() && do_shutdown().is_ok() {
            break;
        }
    }
    len
}

pub fn run_until_shutdown() {
    loop {
        let _tasks = run_until_idle();
        if os_is_shutting_down() && do_shutdown().is_ok() {
            break;
        }
    }
}
