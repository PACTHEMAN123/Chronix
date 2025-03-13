use core::{alloc::Layout, ops::{Deref, DerefMut}, ptr::NonNull, sync::atomic::{AtomicUsize, Ordering}};

use alloc::alloc::{Allocator, Global};

struct StrongArcPayload<T> {
    data: T,
    /// the count of owners = rc + 1
    rc: AtomicUsize,
}

impl<T> StrongArcPayload<T> {
    fn get_rc(&self) -> usize {
        self.rc.load(Ordering::Acquire)
    }
}

/// 只有强引用计数的Arc
#[derive(Debug)]
pub struct StrongArc<
    T: Sized, 
    A: Allocator + Clone = Global,
> {
    payload: NonNull<StrongArcPayload<T>>,
    alloc: A,
}

unsafe impl<T: Sized, A: Allocator + Clone> Send for StrongArc<T, A> {}
unsafe impl<T: Sized, A: Allocator + Clone> Sync for StrongArc<T, A> {}

impl<T: Sized> Clone for StrongArc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (&mut (*self.payload.as_ptr()).rc).fetch_add(1, Ordering::Release);
        }
        Self { payload: self.payload.clone(), alloc: self.alloc.clone()}
    }
}

#[allow(unused, missing_docs)]
impl<T: Sized> StrongArc<T, Global> {
    pub fn new(data: T) -> Self {
        Self::new_in(data, Global)
    }
}

#[allow(unused, missing_docs)]
impl<T: Sized, A: Allocator + Clone> StrongArc<T, A> {
    pub fn new_in(data: T, alloc: A) -> Self {
        match alloc.allocate(Layout::new::<StrongArcPayload<T>>()) {
            Ok(p) => {
                let mut payload: NonNull<StrongArcPayload<T>> = p.cast();
                unsafe {
                    payload.write_volatile(StrongArcPayload {
                        data,
                        rc: AtomicUsize::new(0)
                    });
                }
                Self {
                    payload,
                    alloc,
                }
            },
            Err(_) => panic!("allocate failed")
        }
    }

    pub fn get_rc(&self) -> usize {
        unsafe {
            self.payload.as_ref().get_rc()
        }
    }

    pub fn get_owners(&self) -> usize {
        unsafe {
            self.payload.as_ref().get_rc() + 1
        }
    }
}

impl<T: Sized, A: Allocator + Clone> Deref for StrongArc<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.payload.as_ref().data }
    }
}

impl<T: Sized, A: Allocator + Clone> DerefMut for StrongArc<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.payload.as_mut().data }
    }
}

impl<T: Sized, A: Allocator + Clone> Drop for StrongArc<T, A> {
    fn drop(&mut self) {
        let rc_ref = unsafe {
            &mut self.payload.as_mut().rc
        };
        loop {
            let strong = rc_ref.load(Ordering::Acquire);
            if strong == 0 {
                unsafe { 
                    // self.payload.drop_in_place();
                    self.alloc.deallocate(self.payload.cast(), Layout::new::<StrongArcPayload<T>>());
                }
                self.payload = NonNull::dangling();
                break;
            } else {
                if rc_ref.compare_exchange(strong, strong-1, Ordering::Release, Ordering::Relaxed).is_ok() {
                    break;
                }
            }
        }
    }
}
