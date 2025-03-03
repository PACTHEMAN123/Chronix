//!Implementation of [`TaskControlBlock`]
use super::TaskContext;
use super::{pid_alloc, KernelStack, PidHandle};
use crate::config::{PAGE_SIZE_BITS, TRAP_CONTEXT};
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{PhysPageNum, UserVmSpace, VirtAddr, VmSpace, KERNEL_SPACE, translated_refmut, translated_str, VirtPageNum, VmSpacePageFaultExt, PageFaultAccessType};
use crate::sync::UPSafeCell;
use crate::trap::{trap_handler, TrapContext};
use alloc::slice;
use alloc::{sync::{Arc, Weak}, vec::*, string::String, vec};
use virtio_drivers::PAGE_SIZE;
use core::cell::RefMut;
use core::ptr::slice_from_raw_parts_mut;

use log::*;
use crate::logging;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    #[allow(unused)]
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub vm_space: UserVmSpace,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
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
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // note: the kernel stack must be allocated before the user page table is created
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
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

        let kernel_stack_top = kernel_stack.get_top();
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    vm_space,
                    parent: None,
                    children: Vec::new(),
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
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            kernel_stack_top,
        );
        task_control_block.inner_exclusive_access().get_trap_cx().x[10] = user_sp; // set a0 to user_sp
        task_control_block
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
        for (i, s) in args.iter().map(|s| s.as_bytes()).enumerate() {
            let mut last = s.len() + 1;
            let step = core::cmp::min(last, (data_va - 1) % PAGE_SIZE + 1);
            data_va -= step;
            let pa = translated_refmut(vm_space.token(), data_va as *mut u8) as *mut u8 as usize;

            unsafe { ((pa + last) as *mut u8).write_volatile(0); }
            let dst = unsafe { &mut *slice_from_raw_parts_mut(pa as *mut u8, step - 1) };

            dst.clone_from_slice(&s[last-step..last-1]);
            last -= step;

            while last > 0 {
                let step = core::cmp::min(last, (data_va - 1) % PAGE_SIZE + 1);
                data_va -= step;

                let pa = translated_refmut(vm_space.token(), data_va as *mut u8) as *mut u8 as usize;

                let dst = unsafe { &mut *slice_from_raw_parts_mut(pa as *mut u8, step) };
                dst.clone_from_slice(&s[last-step..last]);
                last -= step;
            }

            meta_data[i+1] = data_va;
        }

        let mut meta_va = new_user_sp + meta_data.len() * SIZE_OF_USIZE;
        {
            let mut last = meta_data.len();
            while last > 0 {
                let step =  core::cmp::min(last * SIZE_OF_USIZE, (meta_va - 1) % PAGE_SIZE + 1);
                meta_va -= step;
                let len = step / SIZE_OF_USIZE;
                let pa = translated_refmut(vm_space.token(), meta_va as *mut usize) as *mut usize as usize;
                let dst = unsafe { &mut *slice_from_raw_parts_mut(pa as *mut usize, len) };
                dst.clone_from_slice(&meta_data[last-len..last]);
                last -= len;
            }
        }

        user_sp = new_user_sp;
        // **** access current TCB exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.vm_space = vm_space;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize trap_cx
        let mut trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            self.kernel_stack.get_top(),
        );
        trap_cx.x[10] = user_sp; // set a0 to user_sp
        *inner.get_trap_cx() = trap_cx;
        // **** release current PCB
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // note: the kernel stack must be allocated before the user page table is created
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        // ---- hold parent PCB lock
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let vm_space = UserVmSpace::from_existed(&mut parent_inner.vm_space);
        let trap_cx_ppn = vm_space
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let kernel_stack_top = kernel_stack.get_top();
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
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    vm_space,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access child PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // **** release child PCB
        // ---- release parent PCB
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
