//!Implementation of [`TaskControlBlock`]
use super::{pid_alloc, schedule, PidHandle, INITPROC};
use crate::config::TRAP_CONTEXT;
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{copy_out, copy_out_str, PhysPageNum, UserVmSpace, VirtAddr, VmSpace, KERNEL_SPACE};
use crate::sync::mutex::spin_mutex::MutexGuard;
use crate::sync::mutex::{MutexSupport, SpinNoIrq, SpinNoIrqLock};
use crate::sync::UPSafeCell;
use crate::trap::{trap_handler, TrapContext};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::{
    cell::RefMut,
    task::Waker,
};
use crate::config::PAGE_SIZE_BITS;
use crate::mm::{ translated_refmut, translated_str, VirtPageNum, VmSpacePageFaultExt, PageFaultAccessType};
use alloc::slice;
use alloc::{vec::*, string::String, };
use virtio_drivers::PAGE_SIZE;
use core::ptr::slice_from_raw_parts_mut;

use log::*;
use crate::logging;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    #[allow(unused)]
    pub base_size: usize,
    pub task_status: TaskStatus,
    pub vm_space: UserVmSpace,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub waker: Option<Waker>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.to_kern().get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.vm_space.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    pub fn is_running(&self) -> bool {
        self.get_status() == TaskStatus::Running
    }
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
    pub fn waker_ref(&self) -> Option<&Waker> {
        self.waker.as_ref()
    }
    pub unsafe fn switch_page_table(&self) {
        self.vm_space.page_table.enable();
    }
    
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> &mut TaskControlBlockInner {
        unsafe {
            self.inner.exclusive_access()
        }
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // note: the kernel stack must be allocated before the user page table is created
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
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
            pid: pid_handle,
            inner: 
            unsafe {
                 UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_status: TaskStatus::Ready,
                    vm_space,
                    parent: None,
                    children: Vec::new(),
                    waker: None,
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                }) 
            } 
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
        );
        task_control_block.inner_exclusive_access().get_trap_cx().x[10] = user_sp; // set a0 to user_sp
        task_control_block
    }
    pub fn set_waker(&self, waker: Waker) {
        unsafe{
            (*self.inner.get()).waker = Some(waker);
        }
    }
    pub fn wake(&self){
        let inner = self.inner_exclusive_access();
        debug_assert!(!(inner.is_zombie() || inner.is_running()));
        let waker = inner.waker_ref();
        waker.unwrap().wake_by_ref();
    }
    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        const SIZE_OF_USIZE: usize = core::mem::size_of::<usize>();
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut vm_space, mut user_sp, entry_point) = UserVmSpace::from_elf(elf_data);
        let trap_cx_ppn = vm_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

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
        // **** access current TCB exclusively
        let inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.vm_space = vm_space;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize trap_cx
        let mut trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
        );
        trap_cx.x[10] = user_sp; // set a0 to user_sp
        *inner.get_trap_cx() = trap_cx;
        // **** release current PCB
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // note: the kernel stack must be allocated before the user page table is created
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        // ---- hold parent PCB lock
        let parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let vm_space = UserVmSpace::from_existed(&mut parent_inner.vm_space);
        let trap_cx_ppn = vm_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            inner: unsafe{
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_status: TaskStatus::Ready,
                    vm_space,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    waker: None,
                    exit_code: 0,
                    fd_table: new_fd_table,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access child PCB exclusively
        // return
        task_control_block
        // **** release child PCB
        // ---- release parent PCB
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn handle_zombie(self: &Arc<Self>){
        let inner = self.inner_exclusive_access();
        for child in inner.children.iter() {
            let initproc_inner = INITPROC.inner_exclusive_access();
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
        inner.children.clear();
        inner.vm_space.recycle_data_pages();
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
