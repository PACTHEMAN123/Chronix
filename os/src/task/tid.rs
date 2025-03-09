//!Implementation of [`PidAllocator`]
use crate::config::{KERNEL_MEMORY_SPACE, KERNEL_STACK_SIZE, PAGE_SIZE};
use crate::mm::{vm::{KernelVmArea, KernelVmAreaType, MapPerm, VmSpace, KERNEL_SPACE}, VirtAddr};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::*;
use crate::sync::mutex::SpinNoIrqLock;

///each task owns unique TaskId
pub type Tid = usize;
/// each process owns unique Pid
pub type Pid = Tid;
/// main thread' tid of a thread group
pub type PGid = Tid;
///Tid Allocator struct
pub struct TidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl TidAllocator {
    ///Create an empty `TidAllocator`
    pub fn new() -> Self {
        TidAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    ///Allocate a tid
    pub fn alloc(&mut self) -> TidHandle {
        if let Some(tid) = self.recycled.pop() {
            TidHandle(tid)  
        } else {
            self.current += 1;
            TidHandle(self.current - 1)
        }
    }
    ///Recycle a pid
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        assert!(
            !self.recycled.iter().any(|ppid| *ppid == pid),
            "pid {} has been deallocated!",
            pid
        );
        self.recycled.push(pid);
    }
}

lazy_static! {
    pub static ref TID_ALLOCATOR: SpinNoIrqLock<TidAllocator> =
    SpinNoIrqLock::new(TidAllocator::new()) ;
}
///Bind pid lifetime to `PidHandle`
pub struct TidHandle(pub usize);

impl Drop for TidHandle {
    fn drop(&mut self) {
        //println!("drop pid {}", self.0);
        TID_ALLOCATOR.lock().dealloc(self.0);
    }
}
///Allocate a pid from PID_ALLOCATOR
pub fn tid_alloc() -> TidHandle {
    TID_ALLOCATOR.lock().alloc()
}

