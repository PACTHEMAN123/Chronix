use core::{cell::UnsafeCell, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

pub struct SyncLazyCell<T> {
    payload: UnsafeCell<T>,
    once_flag: AtomicBool,
}

impl<T> SyncLazyCell<T> {
    pub const fn default() -> Self {
        Self { 
            payload: unsafe {
                core::mem::zeroed()
            }, 
            once_flag: AtomicBool::new(false)
        }
    }

    pub fn emplace(&self, obj: T) -> bool {
        if self.once_flag.compare_exchange(
            false, true, 
            Ordering::Release, Ordering::Relaxed
        ).is_ok() {
            unsafe {
                self.payload.get().write_volatile(obj);
            }
            return true
        }
        false
    }

    pub fn init(&self, init_fn: impl Fn(&mut T)) -> bool {
        if self.once_flag.compare_exchange(
            false, true, 
            Ordering::Release, Ordering::Relaxed
        ).is_ok() {
            unsafe {
                init_fn(&mut *self.payload.get());
            }
            return true;
        }
        false
    }
}

impl<T> Deref for SyncLazyCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if !self.once_flag.load(Ordering::Acquire) {
            panic!("[LazyCell] access a uninit value");
        }
        unsafe {
            &*self.payload.get()
        }
    }
}

impl<T> DerefMut for SyncLazyCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if !self.once_flag.load(Ordering::Acquire) {
            panic!("[LazyCell] access a uninit value");
        }
        self.payload.get_mut()
    }
}

unsafe impl<T> Sync for SyncLazyCell<T> {}
