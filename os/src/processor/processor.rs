//!Implementation of [`Processor`] and Intersection of control flow
use core::arch::asm;
use core::sync::atomic::{AtomicU64, AtomicUsize};
use crate::sync::mutex::SpinNoIrqLock;
use crate::task::task::{get_cpu_mask, new_shared, turn_cpu_mask_to_id, Shared, TaskControlBlock, TaskStatus};
use crate::sync::UPSafeCell;
use crate::processor::context::EnvContext;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use async_task::Runnable;
use hal::instruction::{Instruction, InstructionHal};
use hal::pagetable::PageTableHal;
use hal::println;
use hal::trap::{TrapContext, TrapContextHal};
use crate::mm::vm::KernVmSpaceHal;
use lazy_static::*;
use log::*;
use crate::mm::{self, KVMSPACE};
use hal::board::MAX_PROCESSORS;
const PROCESSOR_OBJECT: Processor = Processor::new();
pub static mut PROCESSORS: [Processor; MAX_PROCESSORS] = [PROCESSOR_OBJECT  ; MAX_PROCESSORS]; 
#[cfg(feature = "smp")]
use super::schedule::TaskLoadTracker;
#[cfg(feature = "smp")]
pub type TaskQueue = VecDeque<Runnable>;
///Processor management structure
pub struct Processor {
    id: usize,
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,
    env: EnvContext,
    #[cfg(feature = "smp")]
    /// each processor has its own task queue
    pub task_queue: Option<Shared<TaskQueue>>,
    #[cfg(feature = "smp")]
    /// counter to decide whether to schedule
    pub counter: AtomicUsize,
    #[cfg(feature = "smp")]
    /// sche_entity for rq
    pub sche_entity: Option<Shared<TaskLoadTracker>>,
    #[cfg(feature = "smp")]
    /// mark whether there is a task need to be migrate
    pub need_migrate: AtomicUsize,
    /// the cpu timeline
    pub timeline: AtomicU64
}
#[cfg(feature = "smp")]
#[macro_export]
/// unwrap first then call the method with the closure
macro_rules! generate_unwrap_with_methods {
    ($($name:ident : $ty:ty),+) => {
        paste::paste! {
            $(
                #[allow(unused)]
                /// with method for Shared<$ty>, takes a closure and returns a reference to the inner value
                pub fn [<unwrap_with_ $name>]<T>(&self, f: impl FnOnce(&$ty) -> T) -> T {
                    log::trace!("with_{}", stringify!($name));
                    f(&self.$name.as_ref().unwrap().lock())
                }
                #[allow(unused)]
                /// with  mut method for Shared<$ty>, takes a closure and returns a mutable reference to the inner value
                pub fn [<unwrap_with_mut_ $name>]<T>(&mut self, f: impl FnOnce(&mut $ty) -> T) -> T {
                    log::trace!("with_mut_{}", stringify!($name));
                    f(&mut self.$name.as_mut().unwrap().lock())
                }
            )+
        }
    };
}

impl Processor {
    ///Create an empty Processor
    pub const fn new() -> Self {
        Self {
            id: 0,
            current: None,
            env: EnvContext::new(),
            #[cfg(feature = "smp")]
            task_queue: None,
            #[cfg(feature = "smp")]
            counter: AtomicUsize::new(0),
            #[cfg(feature = "smp")]
            sche_entity: None,
            timeline: AtomicU64::new(0),
            #[cfg(feature = "smp")]
            need_migrate: AtomicUsize::new(0),
        }
    }
    /// Get the id of the current processor
    pub fn id(&self) -> usize {
        self.id
    }
    /// set the id of the current processor
    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<&Arc<TaskControlBlock>> {
        self.current.as_ref()
    }
    /// Set current task
    pub fn set_current(&mut self, task:Arc<TaskControlBlock>) {
        self.current = Some(task);
    }
    /// judge whether cuurent is None
    pub fn has_current(&self) -> bool {
        self.current.is_some()
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
    #[cfg(feature = "smp")]
    /// set task_queue when first initiated
    pub fn set_task_queue(&mut self) {
        self.task_queue = Some(new_shared(VecDeque::new()));
    }
    #[cfg(feature = "smp")]
    generate_unwrap_with_methods!(
        //task_queue: TaskQueue
        sche_entity: TaskLoadTracker,
        task_queue: TaskQueue
    );
    /// get the num of processor task
    #[cfg(feature = "smp")]
    pub fn task_nums(&self) -> AtomicUsize {
        AtomicUsize::new(self.unwrap_with_task_queue(|q| q.len()))
    }
    #[cfg(feature = "smp")]
    /// initialize sche entity for rq
    pub fn initial_sche_entity(&mut self){
        self.sche_entity = Some(new_shared(TaskLoadTracker::new()));
    }
    #[cfg(feature = "smp")]
    /// get migrate id
    pub fn migrate_id(&self) -> usize {
        self.need_migrate.load(core::sync::atomic::Ordering::SeqCst)
    }
    #[cfg(feature = "smp")]
    /// get need_migrate flag
    pub fn need_migrate_check(&self) -> bool {
        self.need_migrate.load(core::sync::atomic::Ordering::SeqCst) != self.id()
    }
    #[cfg(feature = "smp")]
    /// set need_migrate flag to true
    pub fn set_need_migrate(&mut self, need_migrate: usize) {
        self.need_migrate.store(need_migrate, core::sync::atomic::Ordering::SeqCst);
    } 
    /// get current cpu timeline 
    pub fn get_current_timeline(&self) -> u64 {
        self.timeline.load(core::sync::atomic::Ordering::SeqCst)
    }
    /// adjust current timeline
    pub fn add_current_timeline(&self, added_timeline: u64) {
        let current_timeline = self.get_current_timeline();
        let new_timeline = current_timeline.wrapping_add(added_timeline);
        self.timeline.store(new_timeline, core::sync::atomic::Ordering::SeqCst);
    }
    /**
     * used in following circumstances: 
     * 1. when a task switch out
     * 2. when a task became zombie
     * 3. every time we call rq_task_clock()
     */
    pub fn update_current_timeline(&mut self){
        let current = self.current().unwrap();
        self.add_current_timeline(current.time_recorder().processor_time().as_micros() as u64);
    }
    /// get runqueue task clock
    pub fn rq_task_clock(&mut self) -> u64 {
        self.update_current_timeline();
        self.get_current_timeline()
    }
}
/// current running task of the current processsor
pub fn current_task() -> Option<&'static Arc<TaskControlBlock>> {
    current_processor().current()
}
///Get token of the address space of current task
pub fn current_user_token(processor: &Processor) -> usize {
    let task = processor.current().unwrap();
    let token = task.get_user_token();
    token
}
///Get the mutable reference to trap context of current task
pub fn current_trap_cx(processor: &Processor) -> &mut TrapContext {
    processor.current()
        .unwrap()
        .get_trap_cx()
}

/// Switch to the given task ,change page_table temporarily
pub fn switch_to_current_task(processor: &mut Processor, task: &mut Arc<TaskControlBlock>, env: &mut EnvContext) {
    unsafe{ Instruction::disable_interrupt();}
    unsafe {env.auto_sum();}
    //info!("already in switch");
    #[cfg(feature = "smp")]
    if task.cpu_allowed() != 4 && task.cpu_allowed() != get_cpu_mask(processor.id()) {
        processor.set_need_migrate(turn_cpu_mask_to_id(task.cpu_allowed()));
    }
    processor.set_current(Arc::clone(task));
    #[cfg(feature = "smp")]
    task.set_processor_id(processor.id());
    //info!("[in switch to current task] processor id: {}, task id: {}", processor.id(),task.tid.0);
    task.time_recorder().record_switch_in();
    //info!("[in switch to current task] task id: {}kernel_time:{:?}",task.tid(),task.time_recorder().kernel_time());
    if processor.current().is_none() {
        info!("fail to set current! processor id: {}, task id: {}", processor.id(),task.tid.0);
    }
    core::mem::swap(&mut processor.env, env); 
    //info!("switch page table");
    unsafe {
        task.switch_page_table();
    }
    //info!("switch page table done");
    unsafe{ Instruction::enable_interrupt();}
}

/// Switch out current task,change page_table back to kernel_space
pub fn switch_out_current_task(processor: &mut Processor, env: &mut EnvContext){
    unsafe { Instruction::disable_interrupt()};
    unsafe {env.auto_sum()};
    KVMSPACE.lock().enable();
    core::mem::swap(processor.env_mut(), env);
    let current = processor.current().unwrap();
    current.time_recorder().record_switch_out();
    processor.add_current_timeline(current.time_recorder().processor_time().as_micros() as u64);
    //info!("task id: {}kernel_time:{:?}",current.tid(),current.time_recorder().kernel_time());
    // float_pointer saved, marked restore is needed
    current.get_trap_cx().fx_yield_task();
    processor.current = None;
    unsafe { Instruction::enable_interrupt()};
    //info!("switch_out_current_task done");
}
/// Switch to the kernel task,change sum bit temporarily
pub fn switch_to_current_kernel(processor: &mut Processor, env: &mut EnvContext) {
    unsafe{ Instruction::disable_interrupt();}
    processor.change_env(env);
    core::mem::swap(processor.env_mut(), env);
    unsafe{ Instruction::enable_interrupt()};
}

// multi processor support methods
/// get processor by id
pub fn get_processor(id:usize) -> &'static mut Processor {
    unsafe {&mut PROCESSORS[id]}
}

/// set processor by id and the move tp point to this processor
pub fn set_processor(id:usize) {
    let processor = get_processor(id);
    processor.set_id(id);
    #[cfg(feature = "smp")]
    processor.set_task_queue();
    #[cfg(feature = "smp")]
    processor.initial_sche_entity();
    #[cfg(feature = "smp")]
    processor.set_need_migrate(id);
    let processor_addr = processor as *const _ as usize;
    Instruction::set_tp(processor_addr);
}

/// get current processor
pub fn current_processor() -> &'static mut Processor {
    unsafe {
        &mut *(Instruction::get_tp() as *mut Processor)
    }
} 

pub fn init(id: usize){
    info!("init processor {}", id);
    set_processor(id);
}