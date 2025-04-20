use core::{hash::{BuildHasher, Hasher}, ops::DerefMut, task::Waker};

use alloc::vec::Vec;
use hal::addr::{PhysAddr, VirtAddr};
use hashbrown::HashMap;
use smoltcp::time;

use crate::{mm::{translate_uva_checked, vm::{PageFaultAccessType, UserVmSpaceHal}}, processor::context::SumGuard, signal::{SigSet, SIGKILL, SIGSTOP}, sync::mutex::SpinNoIrqLock, task::{current_task, manager::TASK_MANAGER}, timer::{ffi::TimeSpec, timed_task::suspend_timeout}, utils::{suspend_now, SendWrapper}};

use super::{SysError, SysResult};

/// get futex
#[allow(unused_variables)]
pub async fn sys_futex(
    uaddr: SendWrapper<*const u32>, futex_op: i32, val: u32, 
    timeout: SendWrapper<*const TimeSpec>, uaddr2: SendWrapper<*const u32>, val3: u32
) -> SysResult {
    let mut futex_op = FutexOp::from_bits_truncate(futex_op);
    let task = current_task().unwrap().clone();
    let is_private = futex_op.contains(FutexOp::PRIVATE);
    futex_op.remove(FutexOp::PRIVATE);
    let key = if is_private {
        FutexHashKey::Private {
            mm: task.with_vm_space(|vm| vm as *const _ as usize),
            vaddr: VirtAddr::from(uaddr.0 as usize),
        }
    } else {
        let paddr = task.with_mut_vm_space(|vm| {
            translate_uva_checked(
                vm, 
                VirtAddr::from(uaddr.0 as usize), 
                PageFaultAccessType::WRITE
            ).ok_or(SysError::EINVAL)
        })?;
        FutexHashKey::Shared { paddr }
    };

    match futex_op {
        FutexOp::WAIT => {
            let res = unsafe { 
                let _sum = SumGuard::new();
                uaddr.0.read() 
            };
            if res != val {
                return Err(SysError::EAGAIN);
            }
            futex_manager().add_waiter(
                &key,
                FutexWaiter { 
                    tid: task.tid(), 
                    waker: task.waker().clone().unwrap() 
                } 
            );
            task.set_interruptable();
            let wake_up_sigs = task.with_mut_sig_manager(|sig| {
                sig.wake_sigs = SigSet::from_bits_truncate(!sig.bitmap.bits() | SIGSTOP | SIGKILL); 
                !sig.bitmap
            });
            if timeout.0.is_null() {
                suspend_now().await;
            } else {
                let timeout = unsafe {
                    let _sum = SumGuard::new();
                    timeout.0.read()
                };
                let rem = suspend_timeout(&task, timeout.into()).await;
                if rem.is_zero() {
                    futex_manager().remove_waiter(&key, task.tid());
                }
            }
            if task.with_sig_manager(|sig| sig.bitmap.contains(wake_up_sigs)) {
                log::info!("[sys_futex] Woken by signal");
                futex_manager().remove_waiter(&key, task.tid());
                return Err(SysError::EINTR);
            }
            task.set_running();
            Ok(0)
        }
        FutexOp::WAKE => {
            let n_wake = futex_manager().wake(&key, val)?;
            return Ok(n_wake);
        }
        FutexOp::REQUEUE => {
            let n_wake = futex_manager().wake(&key, val)?;
            let new_key = if is_private {
                FutexHashKey::Private {
                    mm: task.with_vm_space(|vm| vm as *const _ as usize),
                    vaddr: (uaddr2.0 as usize).into(),
                }
            } else {
                let paddr = task.with_mut_vm_space(|vm| {
                    translate_uva_checked(
                        vm, 
                        (uaddr2.0 as usize).into(), 
                        PageFaultAccessType::WRITE
                    ).ok_or(SysError::EINVAL)
                })?;
                FutexHashKey::Shared { paddr }
            };
            let timeout = timeout.0 as usize;
            futex_manager().requeue_waiters(key, new_key, timeout)?;
            Ok(n_wake)
        }
        FutexOp::CMP_REQUEUE => {
            if unsafe {
                let _sum = SumGuard::new();
                uaddr.0.read() as u32 
            }  != val3 {
                return Err(SysError::EAGAIN);
            }
            let n_wake = futex_manager().wake(&key, val)?;
            let new_key = if is_private {
                FutexHashKey::Private {
                    mm: task.with_vm_space(|vm| vm as *const _ as usize),
                    vaddr: (uaddr2.0 as usize).into(),
                }
            } else {
                let paddr = task.with_mut_vm_space(|vm| {
                    translate_uva_checked(
                        vm, 
                        (uaddr2.0 as usize).into(), 
                        PageFaultAccessType::WRITE
                    ).ok_or(SysError::EINVAL)
                })?;
                FutexHashKey::Shared { paddr }
            };
            let timeout = timeout.0 as usize;
            futex_manager().requeue_waiters(key, new_key, timeout)?;
            Ok(n_wake)
        }
        _ => panic!("unimplemented futexop {:?}", futex_op),
    }
}

/// get robust list
#[allow(unused_variables)]
pub fn sys_get_robust_list(
    pid: i32, head_ptr: *mut *const RobustListHead, len_ptr: *mut usize
) -> SysResult {
    let task = if pid != 0 { 
        TASK_MANAGER.get_task(pid as usize).ok_or(SysError::ESRCH)?
    } else {
        current_task().cloned().unwrap()
    };
    if !task.is_leader() {
        return Err(SysError::ESRCH);
    }
    task.with_robust(|&r| {
        unsafe {
            head_ptr.write(r as *const RobustListHead);
            len_ptr.write(size_of::<RobustListHead>());
        }
    });
    Ok(0)
}

/// set robust list
#[allow(unused_variables)]
pub fn sys_set_robust_list(head: *const RobustListHead, len_ptr: usize) -> SysResult {
    if len_ptr != size_of::<RobustListHead>() {
        return Err(SysError::EINVAL);
    }
    let task = current_task().cloned().unwrap();
    task.with_mut_robust(|r| {
        *r = head as usize;
    });
    Ok(0)
}


bitflags::bitflags! {
    /// Futex Operatoion
    pub struct FutexOp: i32 {
        /// Returns 0 if the caller was woken up.  Note that a wake-up
        /// can also be caused by common futex usage patterns in
        /// unrelated code that happened to have previously used the
        /// futex word's memory location (e.g., typical futex-based
        /// implementations of Pthreads mutexes can cause this under
        /// some conditions).  Therefore, callers should always
        /// conservatively assume that a return value of 0 can mean a
        /// spurious wake-up, and use the futex word's value (i.e., the
        /// user-space synchronization scheme) to decide whether to
        /// continue to block or not.
        const WAIT = 0;
        /// Returns the number of waiters that were woken up.
        const WAKE = 1;
        /// Returns the new file descriptor associated with the futex.
        const FD = 2;
        /// Returns the number of waiters that were woken up.
        const REQUEUE = 3;
        /// First checks whether the location uaddr still contains the value
        /// `val3`. If not, the operation fails with the error EAGAIN.
        /// Otherwise, the operation wakes up a maximum of `val` waiters
        /// that are waiting on the futex at `uaddr`. If there are more
        /// than `val` waiters, then the remaining waiters are removed
        /// from the wait queue of the source futex at `uaddr` and added
        /// to the wait queue  of  the  target futex at `uaddr2`.  The
        /// `val2` argument specifies an upper limit on the
        /// number of waiters that are requeued to the futex at `uaddr2`.
        const CMP_REQUEUE = 4;
        /// 
        const WAKE_OP = 5;
        ///
        const LOCK_PI = 6;
        ///
        const UNLOCK_PI = 7;
        ///
        const TRY_LOCK_PI = 8;
        ///
        const WAIT_BITSET = 9;
        ///
        const WAKE_BITSET = 10;
        ///
        const WAIT_BITSET_PI = 11;
        /// Tells the kernel that the futex is process-private and not shared
        /// with another process.
        const PRIVATE = 128;
    }
}

/// futex hash key
#[allow(missing_docs, unused)]
#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Copy, Clone)]
pub enum FutexHashKey {
    Shared { paddr: PhysAddr },
    Private { mm: usize, vaddr: VirtAddr },
}

///
pub static FUTEX_MANAGER: SpinNoIrqLock<FutexManager> =
    SpinNoIrqLock::new(FutexManager::new());

///
pub fn futex_manager() -> impl DerefMut<Target = FutexManager> {
    FUTEX_MANAGER.lock()
}


type Tid = usize;

#[derive(Debug)]
#[allow(missing_docs, unused)]
pub struct FutexWaiter {
    pub tid: Tid,
    pub waker: Waker,
}

#[allow(missing_docs, unused)]
impl FutexWaiter {
    pub fn wake(self) {
        self.waker.wake();
    }
}


///
pub struct XorHasher {
    res: u64,
}

impl Hasher for XorHasher {
    fn finish(&self) -> u64 {
        self.res
    }

    fn write(&mut self, bytes: &[u8]) {
        for x in bytes.chunks(8) {
            let mut t = 0;
            for &i in x {
                t = (t << 8usize) | (i as u64);
            }
            self.res = self.res ^ 31 + t;
        }
    }
}

/// 
pub struct FutexHashKeyBuilder;

impl BuildHasher for FutexHashKeyBuilder {
    type Hasher = XorHasher;

    fn build_hasher(&self) -> Self::Hasher {
        XorHasher { res: 114514 }
    }
}

#[allow(missing_docs, unused)]
pub struct FutexManager {
    futexs: HashMap<FutexHashKey, Vec<FutexWaiter>, FutexHashKeyBuilder>,
}

#[allow(missing_docs, unused)]
impl FutexManager {
    pub const fn new() -> Self {
        Self {
            futexs: HashMap::with_hasher(FutexHashKeyBuilder)
        }
    }

    pub fn add_waiter(&mut self, key: &FutexHashKey, waiter: FutexWaiter) {
        log::info!("[futex::add_waiter] {:?} in {:?} ", waiter, key);
        if let Some(waiters) = self.futexs.get_mut(key) {
            waiters.push(waiter);
        } else {
            let mut waiters = Vec::new();
            waiters.push(waiter);
            self.futexs.insert(*key, waiters);
        }
    }

    /// 用于移除任务，任务可能是过期了，也可能是被信号中断了
    pub fn remove_waiter(&mut self, key: &FutexHashKey, tid: Tid) {
        if let Some(waiters) = self.futexs.get_mut(key) {
            for i in 0..waiters.len() {
                if waiters[i].tid == tid {
                    waiters.swap_remove(i);
                    break;
                }
            }
        }
    }

    pub fn wake(&mut self, key: &FutexHashKey, n: u32) -> SysResult {
        if let Some(waiters) = self.futexs.get_mut(key) {
            let n = core::cmp::min(n as usize, waiters.len());
            for _ in 0..n {
                let waiter = waiters.pop().unwrap();
                log::info!("[futex_wake] {:?} has been woken", waiter);
                waiter.wake();
            }
            Ok(n as isize)
        } else {
            log::debug!("can not find key {key:?}");
            Err(SysError::EINVAL)
        }
    }

    pub fn requeue_waiters(
        &mut self,
        old: FutexHashKey,
        new: FutexHashKey,
        n_req: usize,
    ) -> SysResult {
        let mut old_waiters = self.futexs.remove(&old).ok_or_else(|| {
            log::info!("[futex] no waiters in key {:?}", old);
            SysError::EINVAL
        })?;
        let n = core::cmp::min(n_req as usize, old_waiters.len());
        if let Some(new_waiters) = self.futexs.get_mut(&new) {
            for _ in 0..n {
                new_waiters.push(old_waiters.pop().unwrap());
            }
        } else {
            let mut new_waiters = Vec::with_capacity(n);
            for _ in 0..n {
                new_waiters.push(old_waiters.pop().unwrap());
            }
            self.futexs.insert(new, new_waiters);
        }

        if !old_waiters.is_empty() {
            self.futexs.insert(old, old_waiters);
        }

        Ok(n as isize)
    }
}


/// Robust List Head
#[derive(Clone, Copy, Default)]
#[repr(C)]
#[allow(missing_docs)]
pub struct RobustListHead {
    pub list: usize,
    pub futex_offset: usize,
    pub list_op_pending: usize,
}