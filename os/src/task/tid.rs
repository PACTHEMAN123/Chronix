//!Implementation of [`PidAllocator`]
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

/// Tid address which may be set by `set_tid_address` syscall.
pub struct TidAddress {
    /// When set, when spawning a new thread, the kernel sets the thread's tid
    /// to this address.
    pub set_child_tid: Option<usize>,
    /// When set, when the thread exits, the kernel sets the thread's tid to
    /// this address, and wake up a futex waiting on this address.
    pub clear_child_tid: Option<usize>,
}

impl TidAddress {
    pub const fn new() -> Self {
        Self {
            set_child_tid: None,
            clear_child_tid: None,
        }
    }
}

