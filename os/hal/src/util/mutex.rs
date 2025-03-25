use core::{cell::UnsafeCell, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

use super::sie_guard::SieGuard;

pub struct Mutex<T> {
    val: UnsafeCell<T>,
    mutex: AtomicBool,
}

unsafe impl<T> Sync for Mutex<T> {}
unsafe impl<T> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(val: T) -> Self {
        Self {
            val: UnsafeCell::new(val),
            mutex: AtomicBool::new(false),
        }
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        let mut try_count: usize = 0usize;
        let sie_guard = SieGuard::new();
        core::hint::spin_loop();
        loop {
            if self.mutex.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                break MutexGuard {
                    mutex: self,
                    sie_guard,
                }
            }
            try_count += 1;
            if try_count > 10000000 {
                panic!("dead lock");
            }
        }
    }
}

#[allow(unused)]
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    sie_guard: SieGuard,
}

impl<'a, T> MutexGuard<'a, T> {
    
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.val.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.val.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.mutex.store(false, Ordering::Release);
    }
}

