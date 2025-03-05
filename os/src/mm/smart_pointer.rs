use core::{ops::{Deref, DerefMut, Sub}, ptr::{self, NonNull}, sync::atomic::{AtomicUsize, Ordering}};

use alloc::sync::Arc;
use log::info;

use super::{slab_alloc, slab_dealloc};

#[repr(C)]
pub struct StrongArcPayload<T> {
    rc: AtomicUsize,
    data: T
}

/// 只有强引用计数的Arc
#[derive(Debug)]
pub struct StrongArc<T: Sized> {
    rc: NonNull<AtomicUsize>,
    ptr: NonNull<T>
}

unsafe impl<T: Sized> Send for StrongArc<T> {}
unsafe impl<T: Sized> Sync for StrongArc<T> {}

impl<T: Sized> Clone for StrongArc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (&mut *self.rc.as_ptr()).fetch_add(1, Ordering::Release);
        }
        Self { rc: self.rc.clone(), ptr: self.ptr.clone() }
    }
}

impl<T: Sized> StrongArc<T> {
    pub fn new(data: T) -> Self {
        // 尝试只分配一块连续内存
        match slab_alloc::<StrongArcPayload::<T>>() {
            Some(mut payload) => {
                let payload_ref = unsafe {
                    payload.as_mut()
                };
                payload_ref.rc = AtomicUsize::new(1);
                payload_ref.data = data;
                Self {
                    rc: NonNull::new(&mut payload_ref.rc as *mut AtomicUsize).unwrap(),
                    ptr: NonNull::new(&mut payload_ref.data as *mut T).unwrap(),
                }
            },
            None => {
                let mut ptr = slab_alloc().unwrap();
                unsafe { *ptr.as_mut() = data };
                let mut rc = slab_alloc().unwrap();
                unsafe { *rc.as_mut() = AtomicUsize::new(1) };
                Self {
                    rc,
                    ptr
                }
            }
        }
        
    }

    pub fn from_slab_ptr(data: NonNull<T>) -> Self {
        let mut rc = slab_alloc().unwrap();
        unsafe { *rc.as_mut() = AtomicUsize::new(1) };
        Self {
            rc,
            ptr: data,
        }
    }

    pub fn get_rc(&self) -> usize {
        unsafe { self.rc.as_ref() }.load(Ordering::Acquire)
    }

}

impl<T: Sized> Deref for StrongArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Sized> DerefMut for StrongArc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: Sized> Drop for StrongArc<T> {
    fn drop(&mut self) {
        let rc_ref = unsafe {
            self.rc.as_mut()
        };
        // 自旋
        loop {
            let strong = rc_ref.load(Ordering::Acquire);
            if strong == 1 { // 独占所有权，直接释放
                unsafe { self.ptr.drop_in_place(); };
                if (self.ptr.as_ptr() as usize - self.rc.as_ptr() as usize) <= size_of::<StrongArcPayload<T>>() {
                    // 连续释放
                    slab_dealloc(NonNull::new(self.rc.as_ptr() as usize as *mut StrongArcPayload<T>).unwrap());
                } else {
                    // 分别释放
                    slab_dealloc(self.rc);
                    slab_dealloc(self.ptr);
                }
                // 置为悬空指针
                self.rc = NonNull::dangling();
                self.ptr = NonNull::dangling();
                break;
            } else if strong > 1 {
                // 计数减一
                if rc_ref.compare_exchange(strong, strong-1, Ordering::Release, Ordering::Relaxed).is_ok() {
                    break;
                }
            } else {
                break;
            }
        }
    }
}
