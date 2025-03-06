//!Implementation of [`Processor`] and Intersection of control flow
use super:: TaskStatus;
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use crate::task::{processor, context::EnvContext};
use crate::mm::KERNEL_SPACE;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;
use log::*;
use crate::arch::riscv64::interrupts::{disable_interrupt, enable_interrupt};
use crate::{logging, mm};

///Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,
    env: EnvContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            env: EnvContext::new(),
        }
    }
    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
    /// Get the mutable reference to the environment of the current task
    pub fn env_mut(&mut self) -> &mut EnvContext {
        &mut self.env
    }
    /// get the reference to the environment of the current task
    pub fn env(&self) -> &EnvContext {
        &self.env
    }
    fn change_env(&self, env: &EnvContext) {
        self.env().change_env(env);
    }
}

lazy_static! {
    ///The global processor instance
    pub static ref PROCESSOR: UPSafeCell<Processor> = UPSafeCell::new(Processor::new()) ;
}

///Take the current task,leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}
///Get running task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}
///Get token of the address space of current task
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

/// Switch to the given task ,change page_table temporarily
pub fn switch_to_current_task(task: &mut Arc<TaskControlBlock>, env: &mut EnvContext) {
    unsafe{disable_interrupt();}
    unsafe {env.auto_sum();}
    //info!("already in switch");
    let processor = PROCESSOR.exclusive_access();
    core::mem::swap(&mut processor.env, env);
    processor.current = Some(Arc::clone(task)); 
    let inner = task.inner_exclusive_access();
    //info!("switch page table");
    unsafe {
        inner.switch_page_table();
    }
    //info!("switch page table done");
    //unsafe{enable_interrupt();}
}

/// Switch out current task,change page_table back to kernel_space
pub fn switch_out_current_task(env: &mut EnvContext){
    unsafe {disable_interrupt()};
    unsafe {env.auto_sum()};
    unsafe {
        KERNEL_SPACE.exclusive_access().page_table.enable();
    }
    let processor = PROCESSOR.exclusive_access();
    core::mem::swap(processor.env_mut(), env);
    processor.current = None;
    //unsafe {enable_interrupt()};
    //info!("switch_out_current_task done");
}
/// Switch to the kernel task,change sum bit temporarily
pub fn switch_to_current_kernel(env: &mut EnvContext) {
    unsafe{disable_interrupt();}
    let processor = PROCESSOR.exclusive_access();
    processor.change_env(env);
    core::mem::swap(processor.env_mut(), env);
    //unsafe{enable_interrupt()};
}
