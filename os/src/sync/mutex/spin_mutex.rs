use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering}, usize,
};

use hal::{constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, println};

use crate::{processor::processor::current_processor, utils::async_utils::SendWrapper};
use super::MutexSupport;

/// A spin-lock based mutex.
pub struct MutexGuard<'a, T: ?Sized, S: MutexSupport> {
    mutex: &'a SpinMutex<T, S>,
    support_guard: S::GuardData,
}

/// `SpinMutex` can include different `MutexSupport` type
pub struct SpinMutex<T: ?Sized, S: MutexSupport> {
    owner: AtomicUsize,
    _marker: PhantomData<S>,
    data: UnsafeCell<T>,
}

// Forbid Mutex step over `await` and lead to dead lock
impl<'a, T: ?Sized, S: MutexSupport> !Sync for MutexGuard<'a, T, S> {}
impl<'a, T: ?Sized, S: MutexSupport> !Send for MutexGuard<'a, T, S> {}

unsafe impl<T: ?Sized + Send, S: MutexSupport> Sync for SpinMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: MutexSupport> Send for SpinMutex<T, S> {}

impl<T, S: MutexSupport> SpinMutex<T, S> {
    /// Construct a SpinMutex
    pub const fn new(user_data: T) -> Self {
        SpinMutex {
            owner: AtomicUsize::new(usize::MAX),
            _marker: PhantomData,
            data: UnsafeCell::new(user_data),
        }
    }

    /// Wait until the lock looks unlocked before retrying
    #[inline(always)]
    fn wait_unlock(&self) {
        let mut try_count = 0usize;
        let mut cur_owner = self.owner.load(Ordering::Acquire);
        while cur_owner != usize::MAX {
            if cur_owner >= Constant::MAX_PROCESSORS {
                panic!("owner {:#x} {} > MAX_PROCESSORS", &self.owner as *const _ as usize, cur_owner);
            }
            core::hint::spin_loop();
            try_count += 1;
            if try_count == 0x1000000 {
                panic!("Mutex: deadlock detected! {} try_count > {:#x}, {} is holding lock\n", 
                    Instruction::get_tp(),
                    try_count, 
                    cur_owner, 
                );
            }
            cur_owner = self.owner.load(Ordering::Acquire);
        }
    }

    /// Note that the locked data cannot step over `await`,
    /// i.e. cannot be sent between thread.
    #[inline(always)]
    pub fn lock(&self) -> MutexGuard<T, S> {
        let support_guard = S::before_lock();
        loop {
            let old_owner = self.owner.load(Ordering::Acquire);
            let new_owner = Instruction::get_tp();
            if old_owner == new_owner {
                panic!("[dead lock] hart {} is trying to get the lock, which is already hold by itself", new_owner);
            }
            self.wait_unlock();
            if self
                .owner
                .compare_exchange(usize::MAX, new_owner, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                assert!(new_owner < Constant::MAX_PROCESSORS);
                return MutexGuard {
                    mutex: self,
                    support_guard,
                }
            }
        }
    }

    /// # Safety
    ///
    /// This is highly unsafe.
    /// You should ensure that context switch won't happen during
    /// the locked data's lifetime.
    #[inline(always)]
    pub unsafe fn sent_lock(&self) -> impl DerefMut<Target = T> + '_ {
        SendWrapper::new(self.lock())
    }

    #[inline(always)]
    /// get the inner data
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Deref for MutexGuard<'a, T, S> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> DerefMut for MutexGuard<'a, T, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Drop for MutexGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created
    /// from.
    #[inline(always)]
    fn drop(&mut self) {
        self.mutex.owner.store(usize::MAX, Ordering::Release);
        S::after_unlock(&mut self.support_guard);
    }
}
