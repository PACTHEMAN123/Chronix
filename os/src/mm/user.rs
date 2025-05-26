use core::{fmt::Debug, marker::PhantomData, ops::{Add, Deref, Sub}, ptr::null_mut};

use alloc::sync::Arc;
use hal::{addr::{VirtAddr, VirtAddrHal}, pagetable::MapPerm};

use crate::processor::context::SumGuard;

use super::{vm::{PageFaultAccessType, UserVmSpaceHal}, UserVmSpace};

pub trait UserPtrPerm {}

pub trait UserPtrWrite: UserPtrPerm {}

pub trait UserPtrRead: UserPtrPerm {}

pub trait UserPtrSend: UserPtrPerm {}

#[derive(Debug, Clone, Copy)]
pub struct UserPtrReadMark;

#[derive(Debug, Clone, Copy)]
pub struct UserPtrWriteMark;

#[derive(Debug, Clone, Copy)]
pub struct UserPtrSendReadMark;

#[derive(Debug, Clone, Copy)]
pub struct UserPtrSendWriteMark;

impl UserPtrPerm for UserPtrReadMark {}
impl UserPtrPerm for UserPtrWriteMark{}
impl UserPtrPerm for UserPtrSendReadMark {}
impl UserPtrPerm for UserPtrSendWriteMark{}

impl UserPtrRead for UserPtrReadMark {}
impl UserPtrRead for UserPtrWriteMark {}
impl UserPtrWrite for UserPtrWriteMark {}

impl UserPtrSend for UserPtrSendReadMark {}
impl UserPtrSend for UserPtrSendWriteMark {}
impl UserPtrRead for UserPtrSendReadMark {}
impl UserPtrRead for UserPtrSendWriteMark {}
impl UserPtrWrite for UserPtrSendWriteMark {}

pub type UserPtrReader<T> = UserPtr<T, UserPtrReadMark>;
pub type UserPtrWriter<T> = UserPtr<T, UserPtrWriteMark>;
pub type UserPtrSendReader<T> = UserPtr<T, UserPtrSendReadMark>;
pub type UserPtrSendWriter<T> = UserPtr<T, UserPtrSendWriteMark>;

pub type UserSliceReader<T> = UserSlice<T, UserPtrReadMark>;
pub type UserSliceWriter<T> = UserSlice<T, UserPtrWriteMark>;
pub type UserSliceSendReader<T> = UserSlice<T, UserPtrSendReadMark>;
pub type UserSliceSendWriter<T> = UserSlice<T, UserPtrSendWriteMark>;

#[repr(C)]
#[derive(Clone)]
pub struct UserPtr<T, P: UserPtrPerm> {
    ptr: *mut T,
    _mark: PhantomData<P>,
    _sum_guard: SumGuard,
}

impl<T, P: UserPtrPerm> UserPtr<T, P> {
    /// new user pointer
    pub fn new(ptr: *mut T) -> Self {
        Self {
            ptr,
            _mark: PhantomData,
            _sum_guard: SumGuard::new()
        }
    }

    /// new user pointer, which is null
    /// null is not unvalid
    pub fn null() -> Self {
        Self {
            ptr: null_mut(),
            _mark: PhantomData,
            _sum_guard: SumGuard::new(),
        }
    }

    pub fn reset(&mut self, ptr: *mut T) {
        self.ptr = ptr;
    }

    pub fn cast<T2>(self) -> UserPtr<T2, P> {
        UserPtr {
            ptr: self.ptr as *mut T2,
            _mark: PhantomData,
            _sum_guard: self._sum_guard
        }
    }

    /// get the raw pointer unchecked
    pub unsafe fn to_raw_ptr_unchecked(&self) -> *mut T {
        self.ptr
    }
    
    /// get reference unchecked
    pub unsafe fn to_ref_unchecked(&self) -> & T {
        &*self.ptr
    }

    /// get mutable reference unchecked
    pub unsafe fn to_mut_unchecked(&self) -> &mut T {
        &mut *self.ptr
    }
}

impl<T, P: UserPtrPerm> PartialEq<*mut T> for UserPtr<T, P> {
    fn eq(&self, other: &*mut T) -> bool {
        self.ptr == *other
    }
}

impl<T, P: UserPtrPerm> PartialEq<*const T> for UserPtr<T, P> {
    fn eq(&self, other: &*const T) -> bool {
        self.ptr as *const T == *other
    }
}

impl<T, P: UserPtrPerm> PartialEq<&mut T> for UserPtr<T, P> {
    fn eq(&self, other: &&mut T) -> bool {
        self.ptr as *const _ == (*other) as *const _
    }
}

impl<T, P: UserPtrPerm> PartialEq<&T> for UserPtr<T, P> {
    fn eq(&self, other: &&T) -> bool {
        self.ptr as *const _ == (*other) as *const _
    }
}

impl<T, P: UserPtrPerm> PartialEq for UserPtr<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T, P: UserPtrPerm> Deref for UserPtr<T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.to_ref_unchecked() }
    }
}

impl<T, P: UserPtrPerm> Debug for UserPtr<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserPtr").field("ptr", &self.ptr).finish()
    }
}

impl<T, P: UserPtrPerm> Add<usize> for UserPtr<T, P> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            ptr: unsafe { self.ptr.byte_add(rhs) },
            _mark: PhantomData,
            _sum_guard: SumGuard::new()
        }
    }
}

impl<T, P: UserPtrPerm> Sub<usize> for UserPtr<T, P> {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self {
            ptr: unsafe { self.ptr.byte_sub(rhs) },
            _mark: PhantomData,
            _sum_guard: SumGuard::new()
        }
    }
}

impl<T, P: UserPtrRead> UserPtr<T, P> {
    /// new user pointer
    pub fn new_const(ptr: *const T) -> Self {
        Self {
            ptr: ptr as *mut T,
            _mark: PhantomData,
            _sum_guard: SumGuard::new()
        }
    }

    pub fn to_ref<'a>(&'a self, vm: &mut UserVmSpace) -> Option<&'a T> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>(), PageFaultAccessType::READ).ok()?;
        Some(unsafe { &*self.ptr })
    }
}

impl<T, P: UserPtrWrite> UserPtr<T, P> {
    pub fn to_mut<'a>(&'a self, vm: &mut UserVmSpace) -> Option<&'a mut T> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>(), PageFaultAccessType::WRITE).ok()?;
        Some(unsafe { &mut *self.ptr })
    }
}

unsafe impl<T, P: UserPtrSend> Send for UserPtr<T, P> {}

#[repr(C)]
#[derive(Clone)]
pub struct UserSlice<T, P: UserPtrPerm> {
    len: usize,
    ptr: *mut T,
    _mark: PhantomData<P>,
    _sum_guard: SumGuard,
}

impl<T, P: UserPtrPerm> Debug for UserSlice<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserSlice").field("len", &self.len).field("data", &self.ptr).finish()
    }
}

impl<T, P: UserPtrPerm> UserSlice<T, P> {
    /// from raw part to user slice
    pub fn from_raw_part(ptr: *mut T, len: usize) -> Self {
        Self {
            len,
            ptr,
            _mark: PhantomData,
            _sum_guard: SumGuard::new()
        }
    }

    /// from user pointer to user slice
    pub fn from_user_ptr(ptr: UserPtr<T, impl UserPtrPerm>, len: usize) -> Self {
        Self {
            len,
            ptr: ptr.ptr,
            _mark: PhantomData,
            _sum_guard: SumGuard::new()
        }
    }

    /// get the raw pointer unchecked
    pub unsafe fn to_raw_ptr_unchecked(&self) -> *mut [T] {
        core::slice::from_raw_parts_mut(self.ptr, self.len)
    }
    
    /// get reference unchecked
    pub unsafe fn to_ref_unchecked(&self) -> & [T] {
        &*core::slice::from_raw_parts(self.ptr, self.len)
    }

    /// get mutable reference unchecked
    pub unsafe fn to_mut_unchecked(&self) -> &mut [T] {
        &mut *core::slice::from_raw_parts_mut(self.ptr, self.len)
    }
}

impl<T, P: UserPtrRead> UserSlice<T, P> {
    /// new user pointer
    pub fn new_const(ptr: *const T, len: usize) -> Self {
        Self {
            len,
            ptr: ptr as *mut T,
            _mark: PhantomData,
            _sum_guard: SumGuard::new()
        }
    }

    pub fn to_ref<'a>(&'a self, vm: &mut UserVmSpace) -> Option<&'a [T]> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>()*self.len, PageFaultAccessType::READ).ok()?;
        Some(unsafe { self.to_ref_unchecked() })
    }
}

impl<T, P: UserPtrWrite> UserSlice<T, P> {
    pub fn to_mut<'a>(&'a self, vm: &mut UserVmSpace) -> Option<&'a mut [T]> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>()*self.len, PageFaultAccessType::WRITE).ok()?;
        Some(unsafe { self.to_mut_unchecked() })
    }
}

unsafe impl<T, P: UserPtrSend> Send for UserSlice<T, P> {}