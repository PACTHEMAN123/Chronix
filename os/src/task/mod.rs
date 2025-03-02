//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of [`PidAllocator`] called `PID_ALLOCATOR` allocates
//! pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
/// new task scheduler implementation
pub mod schedule;
mod pid;
pub mod processor;
#[allow(clippy::module_inception)]
#[allow(rustdoc::private_intra_doc_links)]
mod task;

use crate::fs::{open_file, OpenFlags};
use crate::mm::VmSpace;
use crate::sbi::shutdown;
use alloc::sync::Arc;
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};

use log::*;
use crate::logging;

pub use pid::{pid_alloc, KernelStack, PidAllocator, PidHandle};
pub use processor::{
    current_task,  current_user_token,  take_current_task,
    Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // replace by yield_now in async_utils
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task ////and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            //crate::sbi::shutdown(255); //255 == -1 for err hint
            shutdown(true)
        } else {
            //crate::sbi::shutdown(0); //0 for success hint
            shutdown(false)
        }
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.vm_space.recycle_data_pages();
    let _ = inner;
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(inner);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        info!("trying to open initproc");
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}
///Add init process to the manager
pub fn add_initproc() {
    schedule::spawn_user_task(INITPROC.clone());
}

/// init initproc and do nothing
pub fn init_initproc() {
    let _initproc = INITPROC.clone();
}
