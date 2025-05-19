use core::{alloc::Layout, ops::{Deref, DerefMut}, ptr::{self, NonNull}, sync::atomic::{AtomicUsize, Ordering}};

use alloc::{alloc::{handle_alloc_error, Allocator, Global}, rc};

struct StrongArcPayload<T> {
    data: T,
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
    T, 
    A: Allocator + Clone = Global,
> {
    payload: NonNull<StrongArcPayload<T>>,
    alloc: A,
}

unsafe impl<T: Send + Sync, A: Allocator + Clone + Send + Sync> Send for StrongArc<T, A> {}
unsafe impl<T: Send + Sync, A: Allocator + Clone + Send + Sync> Sync for StrongArc<T, A> {}

impl<T, A: Allocator + Clone> Clone for StrongArc<T, A> {
    fn clone(&self) -> Self {
        unsafe {
            self.payload.as_ref().rc.fetch_add(1, Ordering::Release);
        }
        Self { payload: self.payload.clone(), alloc: self.alloc.clone() }
    }
}

#[allow(unused, missing_docs)]
impl<T> StrongArc<T, Global> {
    pub fn new(data: T) -> Self {
        Self::new_in(data, Global)
    }
}

#[allow(unused, missing_docs)]
impl<T, A: Allocator + Clone> StrongArc<T, A> {
    pub fn new_in(data: T, alloc: A) -> Self {
        let mut ret = Self {
            payload: NonNull::dangling(),
            alloc,
        };
        ret.alloc_payload(data);
        ret
    }

    pub fn get_owners(&self) -> usize {
        unsafe {
            self.payload.as_ref().get_rc()
        }
    }

    pub fn emplace(&mut self, val: T) {
        let rc_ref = unsafe { &self.payload.as_ref().rc };
        let mut oval = rc_ref.load(Ordering::Acquire);
        loop {
            if oval == 1 {
                unsafe { 
                    self.payload.as_mut().data = val 
                };
                break;
            } else {
                match rc_ref.compare_exchange(
                    oval, oval-1,
                    Ordering::AcqRel, Ordering::Relaxed
                ) {
                    Ok(_) => {
                        core::sync::atomic::fence(Ordering::Release);
                        self.alloc_payload(val);
                        break;
                    }
                    Err(v) => oval = v
                }
            }
        }
    }

    fn alloc_payload(&mut self, data: T) {
        let layout = Layout::new::<StrongArcPayload<T>>();
        match self.alloc.allocate(layout) {
            Ok(p) => {
                let mut payload: NonNull<StrongArcPayload<T>> = p.cast();
                unsafe {
                    ptr::write(payload.as_ptr(), StrongArcPayload {
                        data,
                        rc: AtomicUsize::new(1)
                    });
                }
                self.payload = payload;
            },
            Err(_) => handle_alloc_error(layout)
        }
    }
}

impl<T, A: Allocator + Clone> Deref for StrongArc<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.payload.as_ref().data }
    }
}

impl<T, A: Allocator + Clone> DerefMut for StrongArc<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.payload.as_mut().data }
    }
}

impl<T, A: Allocator + Clone> Drop for StrongArc<T, A> {
    fn drop(&mut self) {
        let rc_ref = unsafe { &self.payload.as_ref().rc };
        if rc_ref.fetch_sub(1, Ordering::Release) == 1 {
            unsafe {
                core::sync::atomic::fence(Ordering::Acquire);
                ptr::drop_in_place(&mut self.payload.as_mut().data);
                self.alloc.deallocate(
                    self.payload.cast(),
                    Layout::new::<StrongArcPayload<T>>()
                );
            }
        }
    }
}
