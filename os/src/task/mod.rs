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
pub mod task;
/// new task scheduler implementation
pub mod schedule;
mod tid;
/// manger for task
pub mod manager;
pub mod utils;
pub mod fs;
pub mod signal;

#[allow(clippy::module_inception)]
#[allow(rustdoc::private_intra_doc_links)]


use core::sync::atomic::{AtomicI32, Ordering};
use crate::fs::{
    utils::FileReader, vfs::file::open_file, OpenFlags
};
use hal::instruction::{InstructionHal, Instruction};
use alloc::sync::Arc;
use hal::println;
use lazy_static::*;
use manager::{PROCESS_GROUP_MANAGER, TASK_MANAGER};
use task::{TaskControlBlock, TaskStatus};
use log::*;

pub use tid::{tid_alloc, TidAllocator, TidHandle};
pub use crate::processor::processor::{
    current_user_token,current_task,
    Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // replace by yield_now in async_utils
}
/// pid of initproc (must > 0)
pub const INITPROC_PID: usize = 1;

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = {
        //info!("trying to open initproc");
        
        #[cfg(all(target_arch = "riscv64", feature = "autotest"))]
        let file = open_file("/riscv/autotest", OpenFlags::O_WRONLY).unwrap();

        #[cfg(all(target_arch = "riscv64", not(feature = "autotest")))]
        let file = open_file("/riscv/initproc", OpenFlags::O_WRONLY).unwrap();

        #[cfg(all(target_arch = "loongarch64", feature = "autotest"))]
        let file = open_file("/loongarch/autotest", OpenFlags::O_WRONLY).unwrap();

        #[cfg(all(target_arch = "loongarch64", not(feature = "autotest")))]
        let file = open_file("/loongarch/initproc", OpenFlags::O_WRONLY).unwrap();
        

        let reader = FileReader::new(file.clone()).unwrap();
        let elf = xmas_elf::ElfFile::new(&reader).unwrap();
        TaskControlBlock::new(&elf, Some(file)).unwrap()
        // let v = inode.read_all();
        // TaskControlBlock::new(v.as_slice())
    };
}
///Add init process to the manager
pub fn add_initproc() {
    //info!("initproc tid: {}",INITPROC.tid());
    TASK_MANAGER.add_task(&INITPROC);
    PROCESS_GROUP_MANAGER.add_group(&INITPROC);
    schedule::spawn_user_task(INITPROC.clone());
}

/// quick macro to generate with_xxx and with_mut_xxx methods for Shared<T>
#[macro_export]
macro_rules! generate_with_methods {
    ($($name:ident : $ty:ty),+) => {
        paste::paste! {
            $(
                #[allow(unused)]
                /// with method for Shared<$ty>, takes a closure and returns a reference to the inner value
                pub fn [<with_ $name>]<T>(&self, f: impl FnOnce(&$ty) -> T) -> T {
                    log::trace!("with_{}", stringify!($name));
                    f(&self.$name.lock())
                }
                #[allow(unused)]
                /// with  mut method for Shared<$ty>, takes a closure and returns a mutable reference to the inner value
                pub fn [<with_mut_ $name>]<T>(&self, f: impl FnOnce(&mut $ty) -> T) -> T {
                    log::trace!("with_mut_{}", stringify!($name));
                    f(&mut self.$name.lock())
                }
            )+
        }
    };
}

#[macro_export]
macro_rules! generate_option_with_methods {
    ($($name:ident : $ty:ty),+) => {
        paste::paste! {
            $(
                #[allow(unused)]
                /// with method for Shared<$ty>, takes a closure and returns a reference to the inner value
                pub fn [<with_ $name>]<T>(&self, f: impl FnOnce(&$ty) -> T) -> T {
                    log::trace!("with_{}", stringify!($name));
                    f(&self.$name.as_ref().unwrap().lock())
                }
                #[allow(unused)]
                /// with  mut method for Shared<$ty>, takes a closure and returns a mutable reference to the inner value
                pub fn [<with_mut_ $name>]<T>(&self, f: impl FnOnce(&mut $ty) -> T) -> T {
                    log::trace!("with_mut_{}", stringify!($name));
                    f(&mut self.$name.as_ref().unwrap().lock())
                }
            )+
        }
    };
}

#[macro_export]
/// quick macro to generate xxx & set_xxx for SpinNoIrqLock<T>
/// T should be able to Copy, Clone
macro_rules! generate_lock_accessors {
    ($($field_name:ident : $field_type:ty),+) => {
        paste::paste! {
            $(
                /// get the value of the field
                #[allow(unused)]
                pub fn $field_name(&self) -> $field_type {
                    *self.$field_name.lock()
                }
                /// store the value of the field
                #[allow(unused)]
                pub fn [<set_ $field_name>](&self, value: $field_type) {
                    *self.$field_name.lock() = value;
                }
            )+
        }
    };
}

/// quick macro to genrate method to access upsafecell
#[macro_export]
macro_rules! generate_upsafecell_accessors {
    ($($field_name:ident : $field_type:ty),+) => {
        paste::paste! {
            $(
                #[allow(unused)]
                pub fn $field_name(&self) -> &mut $field_type {
                    self.$field_name.exclusive_access()
                }
                #[allow(unused)]
                pub fn [<$field_name _ref>](&self) -> &$field_type {
                    self.$field_name.get_ref()
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
                /// get the value of the field
                #[allow(unused)]
                pub fn $field_name(&self) -> $field_type {
                    self.$field_name.load(Ordering::Relaxed)
                }
                /// store the value of the field
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

