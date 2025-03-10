//!Implementation of [`TaskControlBlock`]
use super::{context, tid_alloc, schedule, INITPROC};
use crate::arch::riscv64::sfence_vma_all;
use crate::config::TRAP_CONTEXT;
use crate::fs::{Stdin, Stdout, vfs::File};
use crate::mm::{copy_out, copy_out_str, PhysPageNum, VirtAddr, VirtPageNum, vm::{UserVmSpace, VmSpace, KERNEL_SPACE}};
use crate::sync::mutex::spin_mutex::MutexGuard;
use crate::sync::mutex::{MutexSupport, SpinNoIrq, SpinNoIrqLock};
use crate::sync::UPSafeCell;
use crate::trap::{trap_handler, TrapContext};
use crate::syscall::process::CloneFlags;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::{fmt, vec};
use alloc::vec::Vec;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicI32, AtomicUsize};
use core::{
    cell::RefMut,
    task::Waker,
};
use crate::config::PAGE_SIZE_BITS;
use crate::mm::{ translated_refmut, translated_str, vm::{VmSpacePageFaultExt, PageFaultAccessType}};
use alloc::slice;
use alloc::{vec::*, string::String, };
use virtio_drivers::PAGE_SIZE;
use core::ptr::slice_from_raw_parts_mut;
use crate::generate_with_methods;
use log::*;
use crate::logging;
use super::context::SumGuard;
use super::tid::{PGid,Pid, Tid, TidHandle};
pub type Shared<T> = Arc<SpinNoIrqLock<T>>;
pub type FDTable = Vec<Option<Arc<dyn File + Send + Sync>>>;
fn new_shared<T>(data: T) -> Shared<T> {
    Arc::new(SpinNoIrqLock::new(data))
}
pub struct TaskControlBlock {
    // immutable
    pub tid: TidHandle,
    pub leader: Option<Weak<TaskControlBlock>>,
    pub is_leader: bool,
    // mutable only in self context , only accessed by current task
    pub trap_cx_ppn: UPSafeCell<PhysPageNum>,
    pub waker: UPSafeCell<Option<Waker>>,
    pub exit_code: AtomicI32,
    #[allow(unused)]
    pub base_size: AtomicUsize,
    // mutable only in self context, can be accessed by other tasks
    pub task_status: SpinNoIrqLock<TaskStatus>,
    // mutable in self and other tasks
    pub vm_space: Shared<UserVmSpace>,
    pub parent: Shared<Option<Weak<TaskControlBlock>>>,
    pub children: Shared<BTreeMap<Pid, Arc<TaskControlBlock>>>,
    pub fd_table: Shared<Vec<Option<Arc<dyn File + Send + Sync>>>>,
    /// thread group which contains this task
    pub thread_group: Shared<ThreadGroup>,
    pub pgid: Shared<PGid>,
}

/// Hold a group of threads which belongs to the same process.
pub struct ThreadGroup {
    members: BTreeMap<Pid, Weak<TaskControlBlock>>,
}

impl ThreadGroup {
    pub fn new() -> Self {
        Self {
            members: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn push(&mut self, task: Arc<TaskControlBlock>) {
        self.members.insert(task.tid(), Arc::downgrade(&task));
    }

    pub fn remove(&mut self, task: &TaskControlBlock) {
        self.members.remove(&task.tid());
    }

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
    generate_with_methods!(
        fd_table: FDTable,
        children: BTreeMap<Pid, Arc<TaskControlBlock>>,
        vm_space: UserVmSpace,
        thread_group: ThreadGroup,
        task_status: TaskStatus
    );
    pub fn pid(self: &Arc<Self>) -> Pid {
        if self.is_leader(){
            self.tid.0
        }
        else {
            self.get_leader().tid.0
        }
    }
    pub fn gettid(&self) -> usize {
        self.tid.0
    }
    pub fn pgid(&self) -> PGid {
        *self.pgid.lock()
    }
    pub fn set_pgid(&self, pgid: PGid) {
        *self.pgid.lock() = pgid
    }
    pub fn tid(&self) -> Tid {
        self.tid.0
    }
    pub fn waker(&self) -> &mut Option<Waker> {
        self.waker.exclusive_access()
    }
    pub fn waker_ref(&self) -> &Option<Waker> {
        self.waker.get_ref()
    }
    pub fn get_trap_cx_ppn_access(&self) -> &mut PhysPageNum {
        self.trap_cx_ppn.exclusive_access()    
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.exclusive_access().to_kern().get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.vm_space.lock().token()
    }
    fn get_status(&self) -> TaskStatus{
        *self.task_status.lock()
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    pub fn is_running(&self) -> bool {
        self.get_status() == TaskStatus::Running
    }
    /// for threads except main thread
    pub fn set_zombie(&self) {
        *self.task_status.lock() = TaskStatus::Zombie;
    }
    pub fn alloc_fd(&self) -> usize {
        let fd_table_inner = self.fd_table.lock();
        if let Some (fd) = (0..fd_table_inner.len()).find(|fd| fd_table_inner[*fd].is_none()) {
            fd
        } else {
            fd_table_inner.len() 
        }
    }
    pub unsafe fn switch_page_table(&self) {
        self.vm_space.lock().page_table.enable();
    }
    pub fn children(&self) -> impl DerefMut<Target = BTreeMap<Tid, Arc<Self>>> + '_ {
        self.children.lock()
    }
    pub fn add_child(&self, child: Arc<TaskControlBlock>) {
        self.children.lock().insert(child.gettid(),child);
    }
    pub fn remove_child(&self, pid: usize) {
        self.children.lock().remove(&pid);
    }
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
    pub fn get_leader(self: &Arc<Self>) -> Arc<Self> {
        if self.is_leader {
            self.clone()
        } else{
            self.leader.as_ref().unwrap().upgrade().unwrap()
        }
    }
    
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8]) -> Self {
        // note: the kernel stack must be allocated before the user page table is created
        // alloc a pid and a kernel stack in kernel space
        let tid_handle = tid_alloc();
        let pgid = tid_handle.0;
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut vm_space, mut user_sp, entry_point) = UserVmSpace::from_elf(elf_data);
        let trap_cx_ppn = vm_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // set argc to zero
        user_sp -= 8;
        vm_space.handle_page_fault(VirtAddr::from(user_sp), PageFaultAccessType::WRITE);
        *translated_refmut(vm_space.token(), user_sp as *mut usize) = 0;

        let task_control_block = Self {
            tid: tid_handle,
            leader: None,
            is_leader: true,
            trap_cx_ppn: UPSafeCell::new(trap_cx_ppn),
            waker: UPSafeCell::new(None),
            exit_code: AtomicI32::new(0),
            base_size: AtomicUsize::new(user_sp),
            task_status: SpinNoIrqLock::new(TaskStatus::Ready),
            vm_space: new_shared(vm_space),
            parent: new_shared(None),
            children:new_shared(BTreeMap::new()),
            fd_table: new_shared(vec![
                // 0 -> stdin
                Some(Arc::new(Stdin)),
                // 1 -> stdout
                Some(Arc::new(Stdout)),
                // 2 -> stderr
                Some(Arc::new(Stdout)),
            ]),
            thread_group: new_shared(ThreadGroup::new()),
            pgid: new_shared(pgid)         
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
        );
        task_control_block.get_trap_cx().x[10] = user_sp; // set a0 to user_sp
        task_control_block
    }
    pub fn set_waker(&self, waker: Waker) {
        unsafe{
            (*self.waker.get()) = Some(waker);
        }
    }
    pub fn wake(&self){
        debug_assert!(!(self.is_zombie() || self.is_running()));
        let waker = self.waker_ref();
        waker.as_ref().unwrap().wake_by_ref();
    }
    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        const SIZE_OF_USIZE: usize = core::mem::size_of::<usize>();
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut vm_space, mut user_sp, entry_point) = UserVmSpace::from_elf(elf_data);
        // update trap_cx ppn
        let trap_cx_ppn = vm_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
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
        unsafe{vm_space.page_table.enable();}
        // todo: close fdtable when exec
        // alloc user resource for main thread again since vm_space has changed
         // push argument to user_stack
        let tot_len: usize = args.iter().map(|s| s.as_bytes().len()+1).sum();
        let new_user_sp = ((user_sp - tot_len) / SIZE_OF_USIZE) * SIZE_OF_USIZE - SIZE_OF_USIZE * (args.len() + 1);
        let frames_num = ((user_sp - new_user_sp) + PAGE_SIZE - 1) / PAGE_SIZE;
        
        for i in 1..frames_num+1 {
            vm_space.handle_page_fault(VirtAddr::from(user_sp - PAGE_SIZE * i), PageFaultAccessType::WRITE);
        }

        let mut meta_data = vec![0usize; args.len()+1];
        meta_data[0] = args.len();

        let mut data_va= user_sp;
        for (i, s) in args.iter().map(|s| s.as_str()).enumerate() {
            data_va -= s.as_bytes().len() + 1;
            copy_out_str(&vm_space.page_table, VirtAddr(data_va), s);
            meta_data[i+1] = data_va;
        }

        copy_out(&vm_space.page_table, VirtAddr(new_user_sp), meta_data.as_slice());

        user_sp = new_user_sp;
        // substitute memory_set
        self.with_mut_vm_space(|m| *m = vm_space);
        // **** access current TCB exclusively
        unsafe {*self.trap_cx_ppn.get() = trap_cx_ppn};
        // initialize trap_cx
        let mut trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
        );
        trap_cx.x[10] = user_sp; // set a0 to user_sp
        *self.get_trap_cx() = trap_cx;
        // **** release current PCB
    }
    pub fn fork(self: &Arc<TaskControlBlock>, flag: CloneFlags) -> Arc<TaskControlBlock> {
        // note: the kernel stack must be allocated before the user page table is created
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
        if flag.contains(CloneFlags::THREAD){
            //info!("creating a thread");
            is_leader = false;
            leader = Some(Arc::downgrade(self));
            parent = self.parent.clone();
            children = self.children.clone();
            thread_group = self.thread_group.clone();
            pgid = self.pgid.clone();
        }
        else{
            is_leader = true;
            leader = None;
            parent =  new_shared(Some(Arc::downgrade(self)));
            children = new_shared(BTreeMap::new());
            thread_group = new_shared(ThreadGroup::new());
            pgid = new_shared(*self.pgid.lock());
        }
        let vm_space;
        if flag.contains(CloneFlags::VM){
            //info!("cloning a vm");
            vm_space = self.vm_space.clone();
        }else {
            vm_space = new_shared(self.with_mut_vm_space(|m| UserVmSpace::from_existed(m)));
            unsafe { sfence_vma_all() };
        }
        let fd_table = if flag.contains(CloneFlags::FILES) {
            //info!("cloning a file descriptor table");
            self.fd_table.clone()
        } else {
            new_shared(self.fd_table.lock().clone())
        };
        let trap_cx_ppn = vm_space
        .lock()
        .translate(VirtAddr::from(TRAP_CONTEXT).into())
        .unwrap()
        .ppn();
        let task_control_block = Arc::new(TaskControlBlock {
            tid: tid_handle,
            leader,
            is_leader,
            trap_cx_ppn: UPSafeCell::new(trap_cx_ppn),
            waker: UPSafeCell::new(None),
            exit_code: AtomicI32::new(0),
            base_size: AtomicUsize::new(0),
            task_status: status,
            vm_space,
            parent,
            children,
            fd_table,
            thread_group,
            pgid,
        });
        // add child except when creating a thread
        if !flag.contains(CloneFlags::THREAD) {
            //info!("fork should in this ");
            self.add_child(task_control_block.clone());
        }
        task_control_block.with_mut_thread_group(|thread_group| thread_group.push(task_control_block.clone()));
        task_control_block
    }
    pub fn handle_zombie(self: &Arc<Self>){
        let mut thread_group = self.thread_group.lock();
        if !self.get_leader().is_zombie() || (self.is_leader && thread_group.len() > 1) || (!self.is_leader && thread_group.len() > 2)
        {
            if !self.is_leader() {
                thread_group.remove(self);
            }
            return;
        }
        if self.is_leader() {
            //info!("therad_group len be {}", thread_group.len());
        }
        else {
            thread_group.remove(self);
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
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
