//!Implementation of [`TaskControlBlock`]
//! 
#![allow(missing_docs)]

use super::fs::FdTable;
use super::manager::{PROCESS_GROUP_MANAGER, TASK_MANAGER};
use super::{tid_alloc, schedule, INITPROC};
use crate::fs::devfs::tty::TTY;
use crate::processor::context::{EnvContext,SumGuard};
use crate::fs::vfs::{Dentry, DCACHE};
use crate::fs::{Stdin, Stdout, vfs::File};
use crate::mm::{copy_out, copy_out_str, UserVmSpace, KVMSPACE};
use crate::processor::processor::{current_processor, PROCESSORS};
#[cfg(feature = "smp")]
use crate::processor::schedule::TaskLoadTracker;
use crate::sync::mutex::spin_mutex::MutexGuard;
use crate::sync::mutex::{MutexSupport, SpinNoIrq, SpinNoIrqLock};
use crate::sync::UPSafeCell;
use crate::syscall::process::CloneFlags;
use crate::signal::{KSigAction, SigManager, SIGKILL, SIGSTOP, SIGCHLD, SigSet};
use crate::syscall::SysError;
use crate::task::utils::user_stack_init;
use crate::timer::get_current_time_duration;
use crate::timer::recoder::TimeRecorder;
use crate::timer::timer::ITimer;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::{fmt, format, task, vec};
use alloc::vec::Vec;
use hal::addr::{PhysAddrHal, PhysPageNum, PhysPageNumHal, VirtAddr, VirtAddrHal, VirtPageNumHal};
use hal::constant::{Constant, ConstantsHal};
use hal::instruction::{Instruction, InstructionHal};
use hal::pagetable::PageTableHal;
use hal::trap::{TrapContext, TrapContextHal};
use hal::println;
use xmas_elf::reader::Reader;
use crate::mm::vm::{self, PageFaultAccessType, UserVmSpaceHal};
use hal::signal::*;
use crate::mm::{ translated_refmut, translated_str};
use alloc::slice;
use alloc::{vec::*, string::String, };
use virtio_drivers::PAGE_SIZE;
use core::any::Any;
use core::arch::global_asm;
use core::ops::Deref;
use core::time::Duration;
use core::{
    ptr::slice_from_raw_parts_mut,
    sync::atomic::{AtomicI32, AtomicUsize, Ordering},
    ops::DerefMut,
    cell::RefMut,
    task::Waker,
};
use crate::{generate_atomic_accessors, generate_state_methods, generate_upsafecell_accessors, generate_with_methods};
use log::*;
use super::tid::{PGid, Pid, Tid, TidAddress, TidHandle};
/// pack Arc<Spin> into a struct
pub type Shared<T> = Arc<SpinNoIrqLock<T>>;
/// new a shared object
pub fn new_shared<T>(data: T) -> Shared<T> {
    Arc::new(SpinNoIrqLock::new(data))
}
/// Task 
pub struct TaskControlBlock {
    // ! immutable
    /// task id
    pub tid: TidHandle,
    /// leader of the thread group
    pub leader: Option<Weak<TaskControlBlock>>,
    /// whether this task is the leader of the thread group
    pub is_leader: bool,
    // ! mutable only in self context , only accessed by current task
    /// trap context physical page number
    pub trap_cx_ppn: UPSafeCell<PhysPageNum>,
    /// waker for waiting on events
    pub waker: UPSafeCell<Option<Waker>>,
    /// address of task's thread ID
    pub tid_address: UPSafeCell<TidAddress>,
    /// time recorder for a task
    pub time_recorder: UPSafeCell<TimeRecorder>,
    // ! mutable only in self context, can be accessed by other tasks
    /// exit code of the task
    pub exit_code: AtomicI32,
    #[allow(unused)]
    /// base address of the user stack, can be used in thread create
    pub base_size: AtomicUsize,
    /// status of the task
    pub task_status: SpinNoIrqLock<TaskStatus>,
    // ! mutable in self and other tasks
    /// virtual memory space of the task
    pub vm_space: Shared<UserVmSpace>,
    /// parent task
    pub parent: Shared<Option<Weak<TaskControlBlock>>>,
    /// child tasks
    pub children: Shared<BTreeMap<Pid, Arc<TaskControlBlock>>>,
    /// file descriptor table
    pub fd_table: Shared<FdTable>,
    /// thread group which contains this task
    pub thread_group: Shared<ThreadGroup>,
    /// process group id
    pub pgid: Shared<PGid>,
    /// use signal manager to handle all the signal
    pub sig_manager: Shared<SigManager>,
    /// pointer to user context for signal handling.
    pub sig_ucontext_ptr: AtomicUsize, 
    /// current working dentry
    pub cwd: Shared<Arc<dyn Dentry>>,
    /// ELF file the task executes
    pub elf: Shared<Option<Arc<dyn File>>>,
    /// Interval timers for the task.
    pub itimers: Shared<[ITimer; 3]>,
    #[cfg(feature = "smp")]
    /// sche_entity of the task
    pub sche_entity: Shared<TaskLoadTracker>,
    #[cfg(feature = "smp")]
    /// the cpu allowed to run this task
    pub cpu_allowed: AtomicUsize,
    #[cfg(feature = "smp")]
    /// the processor id of the task
    pub processor_id: AtomicUsize,
}

/// Hold a group of threads which belongs to the same process.
pub struct ThreadGroup {
    members: BTreeMap<Pid, Weak<TaskControlBlock>>,
}

impl ThreadGroup {
    /// Create a new thread group.
    pub fn new() -> Self {
        Self {
            members: BTreeMap::new(),
        }
    }
    /// Get the number of threads in the group.
    pub fn len(&self) -> usize {
        self.members.len()
    }
    /// Add a task to the group.
    pub fn push(&mut self, task: Arc<TaskControlBlock>) {
        self.members.insert(task.tid(), Arc::downgrade(&task));
    }
    /// Remove a task from the group.
    pub fn remove(&mut self, task: &TaskControlBlock) {
        self.members.remove(&task.tid());
    }
    /// Get an iterator over the tasks in the group.
    pub fn iter(&self) -> impl Iterator<Item = Arc<TaskControlBlock>> + '_ {
        self.members.values().map(|t| t.upgrade().unwrap())
    }
}

impl Drop for TaskControlBlock {
    fn drop(&mut self) {
        info!("Dropping TCB {}", self.tid.0);
    }
}

impl TaskControlBlock {
    generate_upsafecell_accessors!(
        //trap_cx_ppn: PhysPageNum,
        waker: Option<Waker>,
        tid_address: TidAddress,
        time_recorder: TimeRecorder
    );
    generate_with_methods!(
        fd_table: FdTable,
        children: BTreeMap<Pid, Arc<TaskControlBlock>>,
        vm_space: UserVmSpace,
        thread_group: ThreadGroup,
        task_status: TaskStatus,
        sig_manager: SigManager,
        cwd: Arc<dyn Dentry>,
        itimers: [ITimer;3],
        elf: Option<Arc<dyn File>>
    );
    #[cfg(feature = "smp")]
    generate_with_methods!(
        sche_entity: TaskLoadTracker
    );
    generate_atomic_accessors!(
        exit_code: i32,
        sig_ucontext_ptr: usize
    );
    #[cfg(feature = "smp")]
    generate_atomic_accessors!(
        cpu_allowed: usize,
        processor_id: usize
    );
    generate_state_methods!(
        Ready,
        Running,
        Zombie,
        Stopped,
        Terminated,
        Interruptable,
        UnInterruptable
    );
    /// get the process id for a process or leader id for a thread
    pub fn pid(self: &Arc<Self>) -> Pid {
        if self.is_leader(){
            self.tid.0
        }
        else {
            self.get_leader().tid.0
        }
    }
    /// get task id
    pub fn gettid(&self) -> usize {
        self.tid.0
    }
    /// get process group id
    pub fn pgid(&self) -> PGid {
        *self.pgid.lock()
    }
    /// set process group id
    pub fn set_pgid(&self, pgid: PGid) {
        *self.pgid.lock() = pgid
    }
    /// get task id
    pub fn tid(&self) -> Tid {
        self.tid.0
    }
    /// get trap_cx_ppn of the task
    pub fn get_trap_cx_ppn_access(&self) -> &mut PhysPageNum {
        self.trap_cx_ppn.exclusive_access()    
    }
    /// get trap_cx of the task
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.exclusive_access().start_addr().get_mut()
    }
    /// get vm_space of the task
    pub fn get_user_token(&self) -> usize {
        self.vm_space.lock().get_page_table().get_token()
    }
    /// get task_status of the task
    fn get_status(&self) -> TaskStatus{
        *self.task_status.lock()
    }
    /// switch to the task's page table
    pub unsafe fn switch_page_table(&self) {
        self.vm_space.lock().enable();
    }
    /// get parent task
    pub fn parent(&self) -> Option<Weak<Self>> {
        self.parent.lock().clone()
    }
    /// get child tasks
    pub fn children(&self) -> impl DerefMut<Target = BTreeMap<Tid, Arc<Self>>> + '_ {
        self.children.lock()
    }
    /// add a child task
    pub fn add_child(&self, child: Arc<TaskControlBlock>) {
        self.children.lock().insert(child.gettid(),child);
    }
    /// remove a child task
    pub fn remove_child(&self, pid: usize) {
        self.children.lock().remove(&pid);
    }
    /// check whether the task is the leader of the thread group   
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
    /// get the clone of ref of the leader of the thread group
    pub fn get_leader(self: &Arc<Self>) -> Arc<Self> {
        if self.is_leader {
            self.clone()
        } else{
            self.leader.as_ref().unwrap().upgrade().unwrap()
        }
    }
    #[cfg(feature = "smp")]
    pub fn turn_cpu_mask_id(self: Arc<Self>) -> usize {
        let ret = match self.cpu_allowed() {
            1 => 0,
            2 => 1,
            4 => 2,
            8 => 3,
            15 => 4,
            _ => {panic!("cpu_allowed should be 1, 2, 4, 8 or 15")}
        };
        ret
    }
}

impl TaskControlBlock {
    /// new a task with elf data
    pub fn new<T: Reader + ?Sized>(elf: &xmas_elf::ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>) -> Result<Self, SysError> {
        // note: the kernel stack must be allocated before the user page table is created
        // alloc a pid and a kernel stack in kernel space
        let tid_handle = tid_alloc();
        let pgid = tid_handle.0;
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (
            vm_space, 
            mut user_sp, 
            entry_point, 
            _auxv
        ) = UserVmSpace::from_elf(&elf, elf_file.clone())?;

        let trap_cx_ppn = vm_space.get_page_table()
            .translate_vpn(VirtAddr::from(Constant::USER_TRAP_CONTEXT_BOTTOM).floor())
            .unwrap();

        // set argc to zero
        user_sp -= 8;
        // let _ = vm_space.handle_page_fault(VirtAddr::from(user_sp), PageFaultAccessType::WRITE);
        // *translated_refmut(vm_space.get_page_table().get_token(), user_sp as *mut usize) = 0;

        // initproc should set current working dir to root dentry
        let root_dentry = {
            let dcache = DCACHE.lock();
            Arc::clone(dcache.get("/").unwrap())
        };

        let task_control_block = Self {
            tid: tid_handle,
            leader: None,
            is_leader: true,
            trap_cx_ppn: UPSafeCell::new(trap_cx_ppn),
            waker: UPSafeCell::new(None),
            tid_address: UPSafeCell::new(TidAddress::new()),
            time_recorder: UPSafeCell::new(TimeRecorder::new()),
            exit_code: AtomicI32::new(0),
            base_size: AtomicUsize::new(user_sp),
            task_status: SpinNoIrqLock::new(TaskStatus::Ready),
            vm_space: new_shared(vm_space),
            parent: new_shared(None),
            children:new_shared(BTreeMap::new()),
            fd_table: new_shared(FdTable::new()),
            thread_group: new_shared(ThreadGroup::new()),
            pgid: new_shared(pgid),
            sig_manager: new_shared(SigManager::new()),
            sig_ucontext_ptr: AtomicUsize::new(0),
            cwd: new_shared(root_dentry), 
            elf: new_shared(elf_file),
            itimers: new_shared([ITimer::ZERO; 3]),
            #[cfg(feature = "smp")]
            sche_entity: new_shared(TaskLoadTracker::new()),
            #[cfg(feature = "smp")]
            cpu_allowed: AtomicUsize::new(15), 
            #[cfg(feature = "smp")]
            processor_id: AtomicUsize::new(current_processor().id())  
        };
        info!("in new");
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            0,
            0,
            0,
        );
        task_control_block.get_trap_cx().set_arg_nth(0, user_sp); // set a0 to user_sp
        Ok(task_control_block)
    }


    /// 
    pub fn set_waker(&self, waker: Waker) {
        unsafe{
            (*self.waker.get()) = Some(waker);
        }
    }
    /// 
    pub fn wake(&self){
        debug_assert!(!(self.is_zombie() || self.is_running()));
        let waker = self.waker_ref();
        waker.as_ref().unwrap().wake_by_ref();
    }

    /// 
    pub fn exec<T: Reader + ?Sized>(&self, elf: &xmas_elf::ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>, argv: Vec<String>, envp: Vec<String>) ->
        Result<(), SysError> {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (
            mut vm_space, 
            mut user_sp, 
            entry_point, 
            auxv
        ) = UserVmSpace::from_elf(&elf, elf_file.clone())?;
        // update trap_cx ppn
        let trap_cx_ppn = vm_space
            .get_page_table()
            .translate_vpn(VirtAddr::from(Constant::USER_TRAP_CONTEXT_BOTTOM).floor())
            .unwrap();
        // update the executing elf file
        self.with_mut_elf(|elf| *elf = elf_file );
        //  NOTE: should do termination before switching page table, so that other
        // threads will trap in by page fault and be handled by handle_zombie
        //info!("terminating all threads except main");
        let _pid = self.with_thread_group(|thread_group|{
            let mut pid = 0;
            for thread in thread_group.iter() {
                if !thread.is_leader() {
                    thread.set_zombie();
                }else {
                    pid = thread.gettid();
                }
            }
            pid
        });
        //change hart page table
        vm_space.enable();
        // todo: close fdtable when exec
        // alloc user resource for main thread again since vm_space has changed
        // push argument to user_stack
        let (new_user_sp, argc, argv, envp) = user_stack_init(&mut vm_space, user_sp, argv, envp, auxv);
        
        user_sp = new_user_sp;
        // substitute memory_set
        self.with_mut_vm_space(|m| *m = vm_space);
        // **** access current TCB exclusively
        unsafe {*self.trap_cx_ppn.get() = trap_cx_ppn};
        // initialize trap_cx
        let mut trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            argc,
            argv,
            envp,
        );
        //trap_cx.set_arg_nth(0, user_sp); // set a0 to user_sp
        log::debug!("entry: {:x}, argc: {:x}, argv: {:x}, envp: {:x}, sp: {:x}", entry_point, trap_cx.arg_nth(0), trap_cx.arg_nth(1), trap_cx.arg_nth(2), trap_cx.sp());
        *self.get_trap_cx() = trap_cx;
        // **** release current PCB
        Ok(())
    }
    /// 
    pub fn fork(self: &Arc<TaskControlBlock>, flag: CloneFlags) -> Arc<TaskControlBlock> {
        // alloc a pid and a kernel stack in kernel space
        let tid_handle = tid_alloc();
        // ---- hold parent PCB lock
        let status = SpinNoIrqLock::new(self.get_status());
        let leader;
        let is_leader;
        let parent;
        let children;
        let thread_group;
        let pgid;
        let cwd;
        let itimers;
        let sig_manager = new_shared(
            match flag.contains(CloneFlags::SIGHAND) {
            true => SigManager::from_another(&self.sig_manager.lock()),
            false => SigManager::new(),
        });

        if flag.contains(CloneFlags::THREAD){
            //info!("creating a thread");
            is_leader = false;
            leader = Some(Arc::downgrade(self));
            parent = self.parent.clone();
            children = self.children.clone();
            thread_group = self.thread_group.clone();
            pgid = self.pgid.clone();
            cwd = self.cwd.clone();
            itimers = self.itimers.clone();
        }
        else{
            is_leader = true;
            leader = None;
            parent =  new_shared(Some(Arc::downgrade(self)));
            children = new_shared(BTreeMap::new());
            thread_group = new_shared(ThreadGroup::new());
            pgid = new_shared(*self.pgid.lock());
            cwd = new_shared(self.cwd());
            itimers = new_shared([ITimer::ZERO; 3]);
        }
        let vm_space;
        if flag.contains(CloneFlags::VM){
            //info!("cloning a vm");
            vm_space = self.vm_space.clone();
        }else {
            vm_space = new_shared(
                self.with_mut_vm_space(
                    |vm| 
                        UserVmSpace::from_existed(vm)
                )
            );
            unsafe { Instruction::tlb_flush_all() };
        }
        let fd_table = if flag.contains(CloneFlags::FILES) {
            //info!("cloning a file descriptor table");
            self.fd_table.clone()
        } else {
            new_shared(self.fd_table.lock().clone())
        };
        let trap_cx_ppn = vm_space
            .lock()
            .get_page_table()
            .translate_vpn(VirtAddr::from(Constant::USER_TRAP_CONTEXT_BOTTOM).floor())
            .unwrap();
        let task_control_block = Arc::new(TaskControlBlock {
            tid: tid_handle,
            leader,
            is_leader,
            trap_cx_ppn: UPSafeCell::new(trap_cx_ppn),
            waker: UPSafeCell::new(None),
            tid_address: UPSafeCell::new(TidAddress::new()),
            time_recorder: UPSafeCell::new(TimeRecorder::new()),
            exit_code: AtomicI32::new(0),
            base_size: AtomicUsize::new(0),
            task_status: status,
            vm_space,
            parent,
            children,
            fd_table,
            thread_group,
            pgid,
            sig_manager,
            sig_ucontext_ptr: AtomicUsize::new(0),
            cwd,
            elf: self.elf.clone(),
            itimers,
            #[cfg(feature = "smp")]
            sche_entity: new_shared(TaskLoadTracker::new()),
            #[cfg(feature = "smp")]
            cpu_allowed: AtomicUsize::new(15),
            #[cfg(feature = "smp")]
            processor_id: AtomicUsize::new(self.processor_id())
        });
        // add child except when creating a thread
        if !flag.contains(CloneFlags::THREAD) {
            //info!("fork should in this ");
            self.add_child(task_control_block.clone());
        }
        // update user start 
        task_control_block.time_recorder().update_user_start(get_current_time_duration());
        task_control_block.with_mut_thread_group(|thread_group| thread_group.push(task_control_block.clone()));
        if task_control_block.is_leader() {
            PROCESS_GROUP_MANAGER.add_task_to_group(task_control_block.pgid(), &task_control_block);
        }
        TASK_MANAGER.add_task(&task_control_block);
        task_control_block
    }
    /// 
    pub fn handle_zombie(self: &Arc<Self>){
        let mut thread_group = self.thread_group.lock();
        if !self.get_leader().is_zombie() || (self.is_leader && thread_group.len() > 1) || (!self.is_leader && thread_group.len() > 2)
        {
            if !self.is_leader() {
                // for thread, just remove itself from thread_group and task_manager
                thread_group.remove(self);
                TASK_MANAGER.remove_task(self.tid());
            }
            return;
        }
        if self.is_leader() {
            //info!("therad_group len be {}", thread_group.len());
        }
        else {
            thread_group.remove(self);
            TASK_MANAGER.remove_task(self.tid());
        }
        self.with_mut_children(|children|{
            if children.len() == 0 {
                //info!("task {} has no children, should exit", self.tid.0);
                return;
            }
            let initproc = &INITPROC;
            for child in children.values() {
                *child.parent.lock() = Some(Arc::downgrade(initproc));
            }
            initproc.children.lock().extend(children.clone());       
        });
        if self.is_leader() {
            self.set_zombie();
        }else {
            self.get_leader().set_zombie();
        }
        // send signal to parent
        if let Some(parent) = self.parent() {
            //info!("task {} exit, send SIGCHLD to parent", self.pid());
            let parent = parent.upgrade().unwrap();
            parent.recv_sigs(SIGCHLD);
        }
        // set the end time
        self.time_recorder().update_child_time(self.time_recorder().time_pair());
    }
}


/// caculate the process time of a task
impl TaskControlBlock {
    /// get the sum of time pair of all threads in the process 
    pub fn process_time_pair(&self) ->  (Duration, Duration) {
        self.with_thread_group(|thread_group| -> (Duration, Duration) {
            thread_group.iter()
            .map(|thread| thread.time_recorder().time_pair())
            .reduce(|(user_time_one,kernel_time_one),(user_time_two, kernel_time_two)| {
                (user_time_one + user_time_two, kernel_time_one + kernel_time_two)
            })
            .unwrap()
        })
    }
    /// get the sum of user time of all threads in the process
    pub fn process_user_time(&self) -> Duration {
        self.with_thread_group(|thread_group| -> Duration {
            thread_group.iter()
            .map(|thread| thread.time_recorder().user_time())
            .reduce(|time_one, time_two| time_one + time_two)
            .unwrap()
        })
    }
    /// get the sum of cpu_time of all threads in the process
    pub fn process_cpu_time(&self) -> Duration {
        self.with_thread_group(|thread_group| -> Duration{
            thread_group.iter()
            .map(|thread| thread.time_recorder().processor_time())
            .reduce(|time_one, time_two| time_one + time_two)
            .unwrap()
        })
    }
}



#[derive(Copy, Clone, PartialEq)]
/// 
pub enum TaskStatus {
    /// task is ready to run
    Ready,
    /// task is currently running
    Running,
    /// task has terminated for user mode, but hasnt call [exit]
    Terminated,
    /// task has [exit], but the TCB hasnt release
    Zombie,
    /// task has stopped, due to stop signal
    Stopped,
    /// task is waiting for an event
    Interruptable,
    /// task is waiting for an event but cannot be interrupt
    UnInterruptable,
}

bitflags! {
    #[repr(C)]
    pub struct CpuMask: usize {
        const CPU0 = 0b0001;
        const CPU1 = 0b0010;
        const CPU2 = 0b0100;
        const CPU3 = 0b1000;
        const CPU_ALL = 0b1111; 
    }
}
/// a cpum mask converter
pub fn get_cpu_mask(id: usize ) -> usize {
    match id {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 15,
        _ => 0,
    }
}
/// turn a cpu mask to id
pub fn turn_cpu_mask_to_id(mask: usize) -> usize {
    match mask {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        15 => 4,
        _ => 0,
    }
}
