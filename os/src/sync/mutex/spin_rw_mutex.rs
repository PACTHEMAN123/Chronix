use core::{cell::UnsafeCell, marker::PhantomData, ops::{Deref, DerefMut}, sync::atomic::{AtomicUsize, Ordering}};

use hal::println;

use crate::sync::mutex::MutexSupport;

pub struct ReadMutexGuard<'a, T: ?Sized, S: MutexSupport> {
    mutex: &'a SpinRwMutex<T, S>,
    support_guard: S::GuardData,
}

pub struct WriteMutexGuard<'a, T: ?Sized, S: MutexSupport> {
    mutex: &'a SpinRwMutex<T, S>,
    support_guard: S::GuardData,
}


const WRITER_MASK: usize = 1 << (usize::BITS-1);
const READER_MASK: usize = !(1 << (usize::BITS-1));

/// `SpinRwLock` can include different `MutexSupport` type
pub struct SpinRwMutex<T: ?Sized, S: MutexSupport> {
    status: AtomicUsize,
    _marker: PhantomData<S>,
    data: UnsafeCell<T>,
}

impl<T, S: MutexSupport> SpinRwMutex<T, S> {
    pub fn new(user_data: T) -> Self {
        Self { 
            status: AtomicUsize::new(0), 
            _marker: PhantomData, 
            data: UnsafeCell::new(user_data) 
        }
    }
}

impl<T: ?Sized, S: MutexSupport> SpinRwMutex<T, S> {
    #[inline(always)]
    fn wait_unlock_read(&self) {
        let mut try_count = 0usize;
        while self.status.load(Ordering::Acquire) & WRITER_MASK != 0 {
            core::hint::spin_loop();
            try_count += 1;
            if try_count == 0x1000000 {
                panic!("RwMutex: deadlock detected! try_count > {:#x}\n", try_count);
            }
        }
    }

    #[inline(always)]
    fn wait_unlock_write(&self) {
        let mut try_count = 0usize;
        while {
            let status = self.status.load(Ordering::Acquire);
            status & WRITER_MASK != 0 || status & READER_MASK != 0
        } {
            core::hint::spin_loop();
            try_count += 1;
            if try_count == 0x1000000 {
                panic!("RwMutex: deadlock detected! try_count > {:#x}\n", try_count);
            }
        }
    }

    /// Note that the locked data cannot step over `await`,
    /// i.e. cannot be sent between thread.
    #[inline(always)]
    pub fn rlock(&self) -> ReadMutexGuard<T, S> {
        println!("get rlock");
        loop {
            self.wait_unlock_read();
            let oldval = self.status.load(Ordering::Acquire);
            if oldval & READER_MASK == READER_MASK {
                log::warn!("[SpinRwMutex] to many readers");
                continue;
            }
            let support_guard = S::before_lock();
            if self.status.compare_exchange(
                oldval, oldval+1, 
                Ordering::AcqRel, Ordering::Relaxed
            ).is_ok() {
                return ReadMutexGuard {
                    mutex: self,
                    support_guard,
                }
            }
        }
    }

    /// Note that the locked data cannot step over `await`,
    /// i.e. cannot be sent between thread.
    #[inline(always)]
    pub fn wlock(&self) -> WriteMutexGuard<T, S> {
        println!("get wlock");
        loop {
            self.wait_unlock_write();
            let oldval = self.status.load(Ordering::Acquire);
            let support_guard = S::before_lock();
            if self.status.compare_exchange(
                oldval, oldval | WRITER_MASK, 
                Ordering::AcqRel, Ordering::Relaxed
            ).is_ok() {
                return WriteMutexGuard {
                    mutex: self,
                    support_guard,
                }
            }
        }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> !Sync for ReadMutexGuard<'a, T, S> {}
impl<'a, T: ?Sized, S: MutexSupport> !Send for ReadMutexGuard<'a, T, S> {}

impl<'a, T: ?Sized, S: MutexSupport> !Sync for WriteMutexGuard<'a, T, S> {}
impl<'a, T: ?Sized, S: MutexSupport> !Send for WriteMutexGuard<'a, T, S> {}

unsafe impl<T: ?Sized, S: MutexSupport> Sync for SpinRwMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: MutexSupport> Send for SpinRwMutex<T, S> {}

impl<'a, T: ?Sized, S: MutexSupport> ReadMutexGuard<'a, T, S> {

    pub fn upgrade(self) -> Option<WriteMutexGuard<'a, T, S>> {
        let oldval = self.mutex.status.load(Ordering::Acquire);
        if oldval & READER_MASK > 1 || oldval & WRITER_MASK != 0 {
            drop(self);
            return None;
        }
        if self.mutex.status.compare_exchange(
            oldval, WRITER_MASK, 
            Ordering::AcqRel, Ordering::Relaxed
        ).is_ok() {
            let metux = self.mutex;
            let s1 = S::clone(&self.support_guard);
            core::mem::forget(self);
            return Some(WriteMutexGuard {
                mutex: metux,
                support_guard: s1,
            })
        }
        None
    }
}


impl<'a, T: ?Sized, S: MutexSupport> Deref for ReadMutexGuard<'a, T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Deref for WriteMutexGuard<'a, T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> DerefMut for WriteMutexGuard<'a, T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Drop for ReadMutexGuard<'a, T, S> {
    #[inline(always)]
    fn drop(&mut self) {
        self.mutex.status.fetch_sub(1, Ordering::Release);
        S::after_unlock(&mut self.support_guard);
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Drop for WriteMutexGuard<'a, T, S> {
    #[inline(always)]
    fn drop(&mut self) {
        self.mutex.status.fetch_and(!WRITER_MASK, Ordering::Release);
        S::after_unlock(&mut self.support_guard);
    }
}