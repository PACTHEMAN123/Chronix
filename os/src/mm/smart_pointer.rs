use core::{alloc::{GlobalAlloc, Layout}, marker::PhantomData, ops::{Deref, DerefMut, Sub}, ptr::{self, NonNull}, sync::atomic::{AtomicUsize, Ordering}};

use alloc::{alloc::{Allocator, Global}, sync::{Arc, Weak}, task::Wake};
use log::info;

use super::allocator::SlabAllocator;

#[repr(C)]
pub struct StrongArcPayload<T> {
    rc: AtomicUsize,
    data: T
}

pub trait Destructor<T> {
    fn destruct(target: NonNull<T>) {
        unsafe {
            target.drop_in_place();
        }
    }
}

pub struct DefaultDestructor<T> {
    _phantom_data: PhantomData<T>
}

impl<T> Destructor<T> for DefaultDestructor<T> {}

/// 只有强引用计数的Arc
#[derive(Debug)]
pub struct StrongArc<
    T: Sized, 
    A: Allocator + Clone = SlabAllocator,
    D: Destructor<T> = DefaultDestructor<T>,
> {
    rc: NonNull<AtomicUsize>,
    ptr: NonNull<T>,
    alloc: A,
    _phantom_data: PhantomData<D>,
}

unsafe impl<T: Sized, A: Allocator + Clone, D: Destructor<T>> Send for StrongArc<T, A, D> {}
unsafe impl<T: Sized, A: Allocator + Clone, D: Destructor<T>> Sync for StrongArc<T, A, D> {}

impl<T: Sized> Clone for StrongArc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (&mut *self.rc.as_ptr()).fetch_add(1, Ordering::Release);
        }
        Self { rc: self.rc.clone(), ptr: self.ptr.clone(), alloc: self.alloc.clone(), _phantom_data: PhantomData }
    }
}

#[allow(unused, missing_docs)]
impl<T: Sized, D: Destructor<T>> StrongArc<T, SlabAllocator, D> {
    pub fn new(data: T) -> Self {
        Self::new_in(data, SlabAllocator)
    }

    pub fn from_ptr(data: NonNull<T>) -> Self {
        Self::from_ptr_in(data, SlabAllocator)
    }
}

#[allow(unused, missing_docs)]
impl<T: Sized, A: Allocator + Clone, D: Destructor<T>> StrongArc<T, A, D> {
    pub fn new_in(data: T, alloc: A) -> Self {
        // 尝试只分配一块连续内存
        match alloc.allocate(Layout::new::<StrongArcPayload<T>>()) {
            Ok(payload) => {
                let mut payload: NonNull<StrongArcPayload<T>> = payload.cast();
                let payload_ref = unsafe {
                    payload.as_mut()
                };
                payload_ref.rc = AtomicUsize::new(1);
                payload_ref.data = data;
                Self {
                    rc: NonNull::new(&mut payload_ref.rc as *mut AtomicUsize).unwrap(),
                    ptr: NonNull::new(&mut payload_ref.data as *mut T).unwrap(),
                    alloc,
                    _phantom_data: PhantomData
                }
            },
            Err(_) => {
                let mut ptr: NonNull<T> = alloc.allocate(Layout::new::<T>()).unwrap().cast();
                unsafe { *ptr.as_mut() = data };
                let mut rc: NonNull<AtomicUsize> =  alloc.allocate(Layout::new::<AtomicUsize>()).unwrap().cast();
                unsafe { *rc.as_mut() = AtomicUsize::new(1) };
                Self {
                    rc,
                    ptr,
                    alloc,
                    _phantom_data: PhantomData
                }
            }
        }
    }

    pub fn from_ptr_in(data: NonNull<T>, alloc: A) -> Self {
        let mut rc: NonNull<AtomicUsize> = alloc.allocate(Layout::from(Layout::new::<AtomicUsize>())).unwrap().cast();
        unsafe { *rc.as_mut() = AtomicUsize::new(1) };
        Self {
            rc,
            ptr: data,
            alloc,
            _phantom_data: PhantomData
        }
    }

    pub fn get_rc(&self) -> usize {
        unsafe { self.rc.as_ref() }.load(Ordering::Acquire)
    }
}

impl<T: Sized, A: Allocator + Clone, D: Destructor<T>> Deref for StrongArc<T, A, D> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Sized, A: Allocator + Clone, D: Destructor<T>> DerefMut for StrongArc<T, A, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: Sized, A: Allocator + Clone, D: Destructor<T>> Drop for StrongArc<T, A, D> {
    fn drop(&mut self) {
        let rc_ref = unsafe {
            self.rc.as_mut()
        };
        // 自旋
        loop {
            let strong = rc_ref.load(Ordering::Acquire);
            if strong == 1 { // 独占所有权，直接释放
                D::destruct(self.ptr);
                if (self.ptr.as_ptr() as usize - self.rc.as_ptr() as usize) <= size_of::<StrongArcPayload<T>>() {
                    // 连续释放
                    unsafe {
                        self.alloc.deallocate(NonNull::new(self.rc.as_ptr() as usize as *mut u8).unwrap(), Layout::new::<StrongArcPayload<T>>());
                    }
                } else {
                    // 只释放RC
                    unsafe {
                        self.alloc.deallocate(NonNull::new(self.rc.as_ptr() as usize as *mut u8).unwrap(), Layout::new::<AtomicUsize>());
                    }
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

#[allow(unused, missing_docs)]
pub type SlabArc<T> = Arc<T, SlabAllocator>;

#[allow(unused, missing_docs)]
pub type SlabWeak<T> = Weak<T, SlabAllocator>;

#[allow(unused, missing_docs)]
pub trait ArcNewInSlab<T> {
    fn new(data: T) -> Self;
}

impl<T> ArcNewInSlab<T> for SlabArc<T> {
    fn new(data: T) -> Self {
        Self::new_in(data, SlabAllocator)
    }
}