use core::{hash::{BuildHasher, Hasher}, ops::DerefMut, sync::atomic::{AtomicU32, Ordering}, task::Waker, time::Duration};

use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use hal::{addr::{PhysAddr, VirtAddr}, println};
use hashbrown::HashMap;
use log::{info, warn};
use smoltcp::time;

use crate::{mm::{translate_uva_checked, vm::{PageFaultAccessType, UserVmSpaceHal}, UserPtrWriter}, processor::context::SumGuard, signal::{SigSet, SIGKILL, SIGSTOP}, sync::mutex::SpinNoIrqLock, task::{self, current_task, manager::TASK_MANAGER, task::TaskControlBlock}, timer::{self, ffi::TimeSpec, get_current_time_duration, timed_task::suspend_timeout}, utils::{suspend_now, SendWrapper}};

use super::{SysError, SysResult};

const FUTEX_OP_SET: u32 = 0;
const FUTEX_OP_ADD: u32 = 1;
const FUTEX_OP_OR: u32 = 2;
const FUTEX_OP_ANDN: u32 = 3;
const FUTEX_OP_XOR: u32 = 4;
const FUTEX_OP_ARG_SHIFT: u32 = 8;

const FUTEX_OP_CMP_EQ: u32 = 0;
const FUTEX_OP_CMP_NE: u32 = 1;
const FUTEX_OP_CMP_LT: u32 = 2;
const FUTEX_OP_CMP_LE: u32 = 3;
const FUTEX_OP_CMP_GT: u32 = 4;
const FUTEX_OP_CMP_GE: u32 = 5;

fn add_awaiter(fm: &mut FutexManager, task: &Arc<TaskControlBlock>, key: FutexHashKey, mask: u32) {
    task.set_interruptable();
    let wake_up_sigs = task.with_sig_manager(|s| {
        !s.blocked_sigs
    });
    task.set_wake_up_sigs(wake_up_sigs);
    fm.add_waiter(
        &key,
        FutexWaiter { 
            tid: task.tid(), 
            waker: task.waker().clone().unwrap(),
            mask
        } 
    )
}

/// get futex
#[allow(unused_variables)]
pub async fn sys_futex(
    uaddr: usize, mut futex_op: i32, val: u32,
    timeout: SendWrapper<*const TimeSpec>, // or val2: u32
    uaddr2: usize, val3: u32
) -> SysResult {
    let _sum = SumGuard::new();
    let uaddr = unsafe {
        &*(uaddr as *mut AtomicU32)
    };
    let uaddr2 = unsafe {
        &*(uaddr2 as *mut AtomicU32)
    };

    let is_private = futex_op & FUTEX_PRIVATE_FLAG_BITMASK != 0;
    let is_realtime = futex_op & FUTEX_CLOCK_REALTIME_BITMASK != 0;
    futex_op &= !(FUTEX_PRIVATE_FLAG_BITMASK | FUTEX_CLOCK_REALTIME_BITMASK);

    let futex_op = FutexOp::from(futex_op);
    let task = current_task().unwrap().clone();
    
    log::info!("[sys_futex] task {}, futexop {:?}", task.tid(), futex_op);
    let key = if is_private {
        FutexHashKey::Private {
            mm: task.get_raw_vm_ptr(),
            vaddr: VirtAddr::from(uaddr as *const _ as usize),
        }
    } else {
        let paddr = task.with_mut_vm_space(|vm| {
            translate_uva_checked(
                vm, 
                VirtAddr::from(uaddr as *const _ as usize), 
                PageFaultAccessType::WRITE
            ).ok_or(SysError::EINVAL)
        })?;
        FutexHashKey::Shared { paddr }
    };
    
    match futex_op {
        FutexOp::Wait | FutexOp::WaitBitset => {
            log::debug!("[sys_futex] task {} wait at {:?}", task.tid(), key);
            let mask = if futex_op == FutexOp::WaitBitset {
                if val3 == 0 {
                    return Err(SysError::EINVAL);
                } 
                val3
            } else {
                FutexWaiter::FUTEX_BITSET_MATCH_ANY
            };
            
            if timeout.0.is_null() {
                {
                    if uaddr.load(Ordering::Acquire) != val {
                        return Err(SysError::EAGAIN);
                    }
                    // lock futex manager before check
                    let mut fm = futex_manager();
                    if uaddr.load(Ordering::Acquire) != val {
                        return Err(SysError::EAGAIN);
                    }
                    add_awaiter(&mut fm, &task, key, mask);
                }
                suspend_now().await;
            } else {
                let dur;
                {
                    if uaddr.load(Ordering::Acquire) != val {
                        return Err(SysError::EAGAIN);
                    }
                    // lock futex manager before check
                    let mut fm = futex_manager();
                    if uaddr.load(Ordering::Acquire) != val {
                        return Err(SysError::EAGAIN);
                    }
                    add_awaiter(&mut fm, &task, key, mask);
                    let cur = get_current_time_duration();
                    let timeout = unsafe {
                        timeout.0.read()
                    };
                    if !timeout.is_valid() {
                        return Err(SysError::EINVAL);
                    }
                    let timeout: Duration = timeout.into();
                    if is_realtime {
                        if timeout <= cur {
                            task.set_running();
                            if fm.remove_waiter(&key, task.tid()).is_none() {
                                return Ok(0);
                            }
                            log::info!("[sys_futex] Woken by timeout");
                            return Err(SysError::ETIMEOUT);
                        }
                        dur = timeout - cur;
                    } else {
                        dur = timeout;
                    }
                }
                let rem = suspend_timeout(&task, dur).await;
                let mut fm = futex_manager();
                if rem.is_zero() {
                    task.set_running();
                    if fm.remove_waiter(&key, task.tid()).is_none() {
                        return Ok(0);
                    }
                    log::info!("[sys_futex] Woken by timeout");
                    return Err(SysError::ETIMEOUT);
                }
            }
            let mut fm = futex_manager();
            let wake_up_sigs = task.with_sig_manager(|s| {
                    !s.blocked_sigs
                });
            if task.with_sig_manager(|s| s.check_pending_flag(wake_up_sigs)) {
                task.set_running();
                if fm.remove_waiter(&key, task.tid()).is_none() {
                    return Ok(0);
                }
                log::info!("[sys_futex] Woken by signal");
                return Err(SysError::EINTR);
            }
            // log::info!("[sys_futex] woken at {:#x}", uaddr as *const _ as usize);
            task.set_running();
            Ok(0)
        }
        FutexOp::Wake => {
            let n_wake = futex_manager().wake(&key, val)?;
            return Ok(n_wake);
        }
        FutexOp::Fd => {
            return Err(SysError::EINVAL);
        }
        FutexOp::Requeue => {
            let n_woke = futex_manager().wake(&key, val)?;
            let new_key = if is_private {
                FutexHashKey::Private {
                    mm: task.get_raw_vm_ptr(),
                    vaddr: (uaddr2 as *const _ as usize).into(),
                }
            } else {
                let paddr = task.with_mut_vm_space(|vm| {
                    translate_uva_checked(
                        vm, 
                        (uaddr2 as *const _ as usize).into(), 
                        PageFaultAccessType::WRITE
                    ).ok_or(SysError::EINVAL)
                })?;
                FutexHashKey::Shared { paddr }
            };
            // info!("[sys_futex] requeue {:?} to {:?}", key, new_key);
            let timeout = timeout.0 as usize;
            futex_manager().requeue_waiters(key, new_key, timeout)?;
            Ok(n_woke)
        }
        FutexOp::CmpRequeue => {
            if {
                let _sum = SumGuard::new();
                uaddr.load(Ordering::Acquire)
            } != val3 {
                return Err(SysError::EAGAIN);
            }
            let n_woke = futex_manager().wake(&key, val)?;
            let new_key = if is_private {
                FutexHashKey::Private {
                    mm: task.get_raw_vm_ptr(),
                    vaddr: (uaddr2 as *const _ as usize).into(),
                }
            } else {
                let paddr = task.with_mut_vm_space(|vm| {
                    translate_uva_checked(
                        vm, 
                        (uaddr2 as *const _ as usize).into(), 
                        PageFaultAccessType::WRITE
                    ).ok_or(SysError::EINVAL)
                })?;
                FutexHashKey::Shared { paddr }
            };
            let timeout = timeout.0 as usize;
            futex_manager().requeue_waiters(key, new_key, timeout)?;
            Ok(n_woke)
        }
        FutexOp::WakeOp => {
            info!("[sys_futex] wake op");
            let val2 = timeout.0 as u32;
            let op = (val3 >> 28) & 0xF;
            let cmp = (val3 >> 24) & 0xF;
            let oparg = (val3 >> 12) & 0xFFF;
            let cmparg = val3 & 0xFFF;
            
            let actual_oparg = if (op & FUTEX_OP_ARG_SHIFT) != 0 {
                1 << oparg
            } else {
                oparg
            };

            let mut spin_times = 0;
            let mut oldval = uaddr2.load(Ordering::Acquire);
            loop {
                let newval;
                match op & 0x7 {
                    FUTEX_OP_SET => newval = actual_oparg,
                    FUTEX_OP_ADD => newval = oldval.wrapping_add(actual_oparg),
                    FUTEX_OP_OR => newval = oldval | actual_oparg,
                    FUTEX_OP_ANDN => newval = oldval & !actual_oparg,
                    FUTEX_OP_XOR => newval = oldval ^ actual_oparg,
                    _ => panic!("Unknown futex op"),
                };
                match uaddr2.compare_exchange(
                    oldval, newval, 
                    Ordering::AcqRel, Ordering::Relaxed
                ) {
                    Ok(_) => break,
                    Err(v) => oldval = v,
                }
                if spin_times > 100000 {
                    log::warn!("[sys_futex] cas busy");
                    return Err(SysError::EBUSY);
                }
                spin_times += 1;
            }
            let mut fm = futex_manager();
            let n_woke1 = fm.wake(&key, val)?;

            let check = match cmp {
                FUTEX_OP_CMP_EQ => oldval == cmparg,
                FUTEX_OP_CMP_NE => oldval != cmparg,
                FUTEX_OP_CMP_GE => oldval >= cmparg,
                FUTEX_OP_CMP_GT => oldval >  cmparg,
                FUTEX_OP_CMP_LE => oldval <= cmparg,
                FUTEX_OP_CMP_LT => oldval <  cmparg,
                _ => panic!("Unknown futex op cmp"),
            };

            let n_woke2 = if check {
                let key2 = if is_private {
                    FutexHashKey::Private {
                        mm: task.get_raw_vm_ptr(),
                        vaddr: VirtAddr::from(uaddr2 as *const _ as usize),
                    }
                } else {
                    let paddr = task.with_mut_vm_space(|vm| {
                        translate_uva_checked(
                            vm, 
                            VirtAddr::from(uaddr2 as *const _ as usize), 
                            PageFaultAccessType::WRITE
                        ).ok_or(SysError::EINVAL)
                    })?;
                    FutexHashKey::Shared { paddr }
                };
                fm.wake(&key, val2)?
            } else {
                0
            };

            Ok(n_woke1 + n_woke2)
        }
        FutexOp::WakeBitset => {
            if val3 == 0 {
                return Err(SysError::EINVAL);
            }
            let n_wake = futex_manager().wake_bitset(&key, val, val3)?;
            return Ok(n_wake);
        }
        _ => {
            log::warn!("unimplemented futexop {:?}", futex_op);
            Err(SysError::EINVAL)
        }
    }
}

/// Futex Operatoion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FutexOp {
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
    Wait = 0,
    /// Returns the number of waiters that were woken up.
    Wake = 1,
    /// Returns the new file descriptor associated with the futex.
    Fd = 2,
    /// Returns the number of waiters that were woken up.
    Requeue = 3,
    /// First checks whether the location uaddr still contains the value
    /// `val3`. If not, the operation fails with the error EAGAIN.
    /// Otherwise, the operation wakes up a maximum of `val` waiters
    /// that are waiting on the futex at `uaddr`. If there are more
    /// than `val` waiters, then the remaining waiters are removed
    /// from the wait queue of the source futex at `uaddr` and added
    /// to the wait queue  of  the  target futex at `uaddr2`.  The
    /// `val2` argument specifies an upper limit on the
    /// number of waiters that are requeued to the futex at `uaddr2`.
    CmpRequeue = 4,
    /// 
    WakeOp = 5,
    ///
    LockPi = 6,
    ///
    UnlockPi = 7,
    ///
    TryLockPi = 8,
    ///
    WaitBitset = 9,
    ///
    WakeBitset = 10,
    ///
    WaitBitsetPi = 11,
    ///
    Undefined = 12,
}

const FUTEX_PRIVATE_FLAG_BITMASK: i32 = 0x80;
const FUTEX_CLOCK_REALTIME_BITMASK: i32 = 0x100;

impl From<i32> for FutexOp {
    fn from(value: i32) -> Self {
        match value {
            0 => FutexOp::Wait,
            1 => FutexOp::Wake,
            2 => FutexOp::Fd,
            3 => FutexOp::Requeue,
            4 => FutexOp::CmpRequeue,
            5 => FutexOp::WakeOp,
            6 => FutexOp::LockPi,
            7 => FutexOp::UnlockPi,
            8 => FutexOp::TryLockPi,
            9 => FutexOp::WaitBitset,
            10 => FutexOp::WakeBitset,
            11 => FutexOp::WaitBitsetPi,
            _ => FutexOp::Undefined,
        }
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
    pub mask: u32,
}

#[allow(missing_docs, unused)]
impl FutexWaiter {

    const FUTEX_BITSET_MATCH_ANY: u32 = 0xFFFF_FFFF;

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
    futexs: HashMap<FutexHashKey, VecDeque<FutexWaiter>, FutexHashKeyBuilder>,
}

#[allow(missing_docs, unused)]
impl FutexManager {
    pub const fn new() -> Self {
        Self {
            futexs: HashMap::with_hasher(FutexHashKeyBuilder)
        }
    }

    pub fn add_waiter(&mut self, key: &FutexHashKey, waiter: FutexWaiter) {
        // log::info!("[futex::add_waiter] {:?} in {:?} ", waiter, key);
        if let Some(waiters) = self.futexs.get_mut(key) {
            waiters.push_back(waiter);
        } else {
            let mut waiters = VecDeque::with_capacity(1);
            waiters.push_back(waiter);
            self.futexs.insert(*key, waiters);
        }
    }

    /// 用于移除任务，任务可能是过期了，也可能是被信号中断了
    pub fn remove_waiter(&mut self, key: &FutexHashKey, tid: Tid) -> Option<FutexWaiter> {
        if let Some(waiters) = self.futexs.get_mut(key) {
            for i in 0..waiters.len() {
                if waiters[i].tid == tid {
                    return waiters.remove(i);
                }
            }
        }
        None
    }

    pub fn wake(&mut self, key: &FutexHashKey, n: u32) -> SysResult {
        if let Some(waiters) = self.futexs.get_mut(key) {
            let n = core::cmp::min(n as usize, waiters.len());
            for _ in 0..n {
                let waiter = waiters.pop_front().unwrap();
                log::debug!("[futex_wake] task {} has been woken at {:?}", waiter.tid, key);
                waiter.wake();
            }
            Ok(n as isize)
        } else {
            log::debug!("can not find key {key:?}");
            Err(SysError::EINVAL)
        }
    }

    pub fn wake_bitset(&mut self, key: &FutexHashKey, n: u32, mask: u32) -> SysResult {
        if let Some(waiters) = self.futexs.get_mut(key) {
            let mut count = 0;
            let max_count = n as usize;

            let mut i = 0;
            let len = waiters.len();
            while i < len && count < max_count {
                if (waiters[len - 1 - i].mask & mask) != 0 {
                    let waiter = waiters.remove(i).unwrap();
                    // log::info!("[futex_wake] {:?} has been woken", waiter);
                    waiter.wake();
                    count += 1;
                } else {
                    i += 1;
                }
            }
            
            Ok(count as isize)
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
                new_waiters.push_back(old_waiters.pop_front().unwrap());
            }
        } else {
            let mut new_waiters = VecDeque::with_capacity(n);
            for _ in 0..n {
                new_waiters.push_back(old_waiters.pop_front().unwrap());
            }
            self.futexs.insert(new, new_waiters);
        }

        if !old_waiters.is_empty() {
            self.futexs.insert(old, old_waiters);
        }

        Ok(n as isize)
    }
}

/// Per-lock list entry - embedded in user-space locks, somewhere close
/// to the futex field. (Note: user-space uses a double-linked list to
/// achieve O(1) list add and remove, but the kernel only needs to know
/// about the forward link)
/// 
/// NOTE: this structure is part of the syscall ABI, and must not be
/// changed.
#[derive(Clone)]
#[repr(C)]
pub struct RobustList {
    ///
	pub next: UserPtrWriter<RobustList>
}

/// Robust List Head
#[derive(Clone)]
#[repr(C)]
pub struct RobustListHead {
    /// The head of the list. Points back to itself if empty:
    pub list: RobustList,
    /// This relative offset is set by user-space, it gives the kernel
	/// the relative position of the futex field to examine. This way
	/// we keep userspace flexible, to freely shape its data-structure,
	/// without hardcoding any particular offset into the kernel:
    pub futex_offset: usize,
	/// The death of the thread may race with userspace setting
	/// up a lock's links. So to handle this race, userspace first
	/// sets this field to the address of the to-be-taken lock,
	/// then does the lock acquire, and then adds itself to the
	/// list, and then clears this field. Hence the kernel will
	/// always have full knowledge of all locks that the thread
	/// _might_ have taken. We check the owner TID in any case,
	/// so only truly owned locks will be handled.
    pub list_op_pending: UserPtrWriter<RobustList>,
}

/// Are there any waiters for this robust futex:
#[allow(unused)]
pub const FUTEX_WAITERS: u32 = 0x8000_0000;

/// The kernel signals via this bit that a thread holding a futex
/// has exited without unlocking the futex. The kernel also does
/// a FUTEX_WAKE on such futexes, after setting the bit, to wake
/// up any possible waiters:
pub const FUTEX_OWNER_DIED: u32 = 0x4000_0000;

/// The rest of the robust-futex field is for the TID:
pub const FUTEX_TID_MASK: u32 = 0x3fff_ffff;

/// get robust list
#[allow(unused_variables)]
pub fn sys_get_robust_list(
    pid: i32, head_ptr: *mut *mut RobustListHead, len_ptr: *mut usize
) -> SysResult {
    let task = if pid != 0 { 
        TASK_MANAGER.get_task(pid as usize).ok_or(SysError::ESRCH)?
    } else {
        current_task().cloned().unwrap()
    };
    unsafe {
        head_ptr.write(task.robust.exclusive_access().to_raw_ptr_unchecked());
        len_ptr.write(size_of::<RobustListHead>());
    }
    Ok(0)
}

/// set robust list
#[allow(unused_variables)]
pub fn sys_set_robust_list(head: *mut RobustListHead, len: usize) -> SysResult {
    if len != size_of::<RobustListHead>() {
        return Err(SysError::EINVAL);
    }
    let task = current_task().cloned().unwrap();
    // task.vm_space.lock().get_area_mut(VirtAddr::from(head as usize)).ok_or(SysError::EINVAL)?;
    info!("[sys_set_robust_list] set task {} robust to {:#x}", task.tid(), head as usize);
    task.robust.exclusive_access().reset(head);
    Ok(0)
}
