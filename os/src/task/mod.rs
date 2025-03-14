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
mod tid;
pub mod processor;
#[allow(clippy::module_inception)]
#[allow(rustdoc::private_intra_doc_links)]
mod task;

use core::sync::atomic::{AtomicI32, Ordering};
use crate::fs::{
    ext4::open_file,
    OpenFlags,
};
use crate::mm::vm::VmSpace;
use crate::sbi::shutdown;
use alloc::sync::Arc;
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};

use log::*;
use crate::logging;

pub use tid::{tid_alloc, TidAllocator, TidHandle};
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
pub fn exit_current_and_run_next(exit_code: i32)  {
    // take from Processor
    let task = current_task().unwrap();
    task.exit_code.store(exit_code, Ordering::Relaxed);
    let tid = task.gettid();
    println!("[kernel] Task {} exit with exit_code {} ...", tid, exit_code);
    if tid == IDLE_PID {
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
    // Change status to Zombie
    //info!("now set task {} status to Zombie", task.tid());
    task.with_mut_task_status(|state| *state = TaskStatus::Zombie);
    // do not move to its parent but under initproc
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        //info!("trying to open initproc");
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}
///Add init process to the manager
pub fn add_initproc() {
    //info!("initproc tid: {}",INITPROC.tid());
    schedule::spawn_user_task(INITPROC.clone());
}

/// quick macro to generate with_xxx and with_mut_xxx methods for Shared<T>
#[macro_export]
macro_rules! generate_with_methods {
    ($($name:ident : $ty:ty),+) => {
        paste::paste! {
            $(
                #[allow(unused)]
                pub fn [<with_ $name>]<T>(&self, f: impl FnOnce(&$ty) -> T) -> T {
                    log::trace!("with_{}", stringify!($name));
                    f(&self.$name.lock())
                }
                #[allow(unused)]
                pub fn [<with_mut_ $name>]<T>(&self, f: impl FnOnce(&mut $ty) -> T) -> T {
                    log::trace!("with_mut_{}", stringify!($name));
                    f(&mut self.$name.lock())
                }
            )+
        }
    };
}

/// quick macro to generate set_xxx for AtomUsizes
#[macro_export]
macro_rules! generate_atomic_accessors {
    ($($field_name:ident : $field_type:ty),+) => {
        paste::paste! {
            $(
                #[allow(unused)]
                pub fn $field_name(&self) -> $field_type {
                    self.$field_name.load(Ordering::Relaxed)
                }
                #[allow(unused)]
                pub fn [<set_ $field_name>](&self, value: $field_type) {
                    self.$field_name.store(value, Ordering::Relaxed);
                }
            )+
        }
    };
}

/// quick macro to generate is_xxx and set_xxx state method
#[macro_export]
macro_rules! generate_state_methods {
    ($($state:ident),+) => {
        $(
            paste::paste! {
                #[allow(unused)]
                pub fn [<is_ $state:lower>](&self) -> bool {
                    *self.task_status.lock() == TaskStatus::$state
                }
                #[allow(unused)]
                pub fn [<set_ $state:lower>](&self) {
                    *self.task_status.lock() = TaskStatus::$state
                }
            }
        )+
    };
}

