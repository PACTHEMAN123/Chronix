use core::{fmt::Debug, marker::PhantomData, ops::{Add, Deref, DerefMut, Sub}, ptr::null_mut, slice, str};

use alloc::sync::Arc;
use hal::{addr::{VirtAddr, VirtAddrHal}, constant::{Constant, ConstantsHal}, pagetable::MapPerm};

use crate::{mm::vm::UserVmPagesLocker, processor::context::SumGuard, sync::mutex::{spin_mutex::MutexGuard, SpinNoIrq}};

use super::{vm::{PageFaultAccessType, UserVmSpaceHal}, UserVmSpace};

pub trait UserPtrPerm {}

pub trait UserPtrWrite: UserPtrPerm {}

pub trait UserPtrRead: UserPtrPerm {}

#[derive(Debug, Clone, Copy)]
pub struct ReadMark;

#[derive(Debug, Clone, Copy)]
pub struct WriteMark;

impl UserPtrPerm for ReadMark {}
impl UserPtrPerm for WriteMark{}

impl UserPtrRead for ReadMark {}
impl UserPtrRead for WriteMark {}
impl UserPtrWrite for WriteMark {}


#[repr(C)]
pub struct UserPtrRaw<T> {
    ptr: *mut T
}

impl<T> Debug for UserPtrRaw<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserPtrRaw").field("ptr", &self.ptr).finish()
    }
}

impl<T> Clone for UserPtrRaw<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Copy for UserPtrRaw<T> {}

impl<T> UserPtrRaw<T> {
    /// new user pointer
    pub fn new(ptr: *const T) -> Self {
        Self {
            ptr: ptr as *mut T
        }
    }

    pub fn cast<T2>(self) -> UserPtrRaw<T2> {
        UserPtrRaw {
            ptr: self.ptr as *mut T2
        }
    }

    pub fn ensure_read(self, vm: &mut UserVmSpace) -> Option<UserPtr<T, ReadMark>> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>(), PageFaultAccessType::READ).ok()?;
        Some(UserPtr { raw: self, _mark: PhantomData, _sum_guard: SumGuard::new(), locker: UserVmPagesLocker {  } })
    }

    pub fn ensure_write(self, vm: &mut UserVmSpace) -> Option<UserPtr<T, WriteMark>> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>(), PageFaultAccessType::WRITE).ok()?;
        Some(UserPtr { raw: self, _mark: PhantomData, _sum_guard: SumGuard::new(), locker: UserVmPagesLocker {  }  })
    }

    pub fn reset(&mut self, ptr: *mut T) {
        self.ptr = ptr;
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

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl UserPtrRaw<u8> {
    pub fn cstr_slice(self, vm: &mut UserVmSpace) -> Option<UserSlice<u8, ReadMark>> {
        let sum_guard = SumGuard::new();
        let mut cur = self.ptr;
        let mut len = 0;
        loop {
            vm.ensure_access((cur as usize).into(), 1, PageFaultAccessType::READ).ok()?;
            let pg_end = ((cur as usize + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE - 1)) as *mut u8;
            while cur != pg_end {
                if unsafe { *cur != 0u8 } {
                    len += 1;
                } else {
                    return Some(UserSlice {
                        raw: UserSliceRaw { len, ptr: self.ptr },
                        _mark: PhantomData,
                        _sum_guard: sum_guard,
                        locker: UserVmPagesLocker { },
                    });
                }
                cur = unsafe { cur.add(1) };
            }
            cur = pg_end;
        }
    }
}

impl<T> From<*mut T> for UserPtrRaw<T> {
    fn from(value: *mut T) -> Self {
        Self { ptr: value } 
    }
}

impl<T> From<*const T> for UserPtrRaw<T> {
    fn from(value: *const T) -> Self {
        Self { ptr: value as *mut T } 
    }
}

impl<T> PartialEq<*mut T> for UserPtrRaw<T> {
    fn eq(&self, other: &*mut T) -> bool {
        self.ptr == *other
    }
}

impl<T> PartialEq<*const T> for UserPtrRaw<T> {
    fn eq(&self, other: &*const T) -> bool {
        self.ptr as *const T == *other
    }
}

impl<T> PartialEq<&mut T> for UserPtrRaw<T> {
    fn eq(&self, other: &&mut T) -> bool {
        self.ptr as *const _ == (*other) as *const _
    }
}

impl<T> PartialEq<&T> for UserPtrRaw<T> {
    fn eq(&self, other: &&T) -> bool {
        self.ptr as *const _ == (*other) as *const _
    }
}

impl<T> PartialEq for UserPtrRaw<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Add<usize> for UserPtrRaw<T> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            ptr: unsafe { self.ptr.add(rhs) },
        }
    }
}

impl<T> Sub<usize> for UserPtrRaw<T> {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self {
            ptr: unsafe { self.ptr.sub(rhs) },
        }
    }
}

unsafe impl<T> Send for UserPtrRaw<T> {}

#[repr(C)]
#[derive(Clone)]
pub struct UserPtr<T, P: UserPtrPerm> {
    pub raw: UserPtrRaw<T>,
    _mark: PhantomData<P>,
    _sum_guard: SumGuard,
    locker: UserVmPagesLocker
}

impl<T, P: UserPtrPerm> Deref for UserPtr<T, P>  {
    type Target = UserPtrRaw<T>;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<'a, T, P: UserPtrPerm> DerefMut for UserPtr<T, P>  {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

impl<T, P: UserPtrPerm> UserPtr<T, P> {
    pub fn cast<T2>(self) -> UserPtr<T2, P> {
        UserPtr { raw: self.raw.cast(), _mark: PhantomData, _sum_guard: self._sum_guard, locker: self.locker }
    }

    pub unsafe fn cast_perm<P2: UserPtrPerm>(self) -> UserPtr<T, P2> {
        UserPtr { raw: self.raw, _mark: PhantomData, _sum_guard: self._sum_guard, locker: self.locker }
    }
}


impl<T, P: UserPtrPerm> Debug for UserPtr<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserPtr").field("raw", &self.raw).finish()
    }
}

impl<T, P: UserPtrRead> UserPtr<T, P> {
    pub fn to_ref(&self) -> &T {
        unsafe { &*self.raw.ptr }
    }
}

impl<T, P: UserPtrWrite> UserPtr<T, P> {
    pub fn to_read(self) -> Option<UserPtr<T, ReadMark>> {
        Some(UserPtr { raw: self.raw, _mark: PhantomData, _sum_guard: self._sum_guard, locker: self.locker })
    }

    pub fn to_mut(&self) -> &mut T {
        unsafe { &mut *self.raw.ptr }
    }

    pub fn write(&self, val: T) {
        unsafe {
            self.raw.ptr.write(val);
        }
    }
}

unsafe impl<T, P: UserPtrPerm> Send for UserPtr<T, P> {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UserSliceRaw<T> {
    len: usize,
    ptr: *mut T,
}

impl<T> Debug for UserSliceRaw<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserSliceRaw").field("len", &self.len).field("ptr", &self.ptr).finish()
    }
}

impl<T> UserSliceRaw<T> {
    /// new user pointer
    pub fn new(ptr: *const T, len: usize) -> Self {
        Self {
            len,
            ptr: ptr as *mut T,
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

    pub fn ensure_read(self, vm: &mut UserVmSpace) -> Option<UserSlice<T, ReadMark>> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>()*self.len, PageFaultAccessType::READ).ok()?;
        Some(UserSlice { raw: self, _mark: PhantomData, _sum_guard: SumGuard::new(), locker: UserVmPagesLocker {  } })
    }

    pub fn ensure_write(self, vm: &mut UserVmSpace) -> Option<UserSlice<T, WriteMark>> {
        let va = VirtAddr(self.ptr as usize);
        vm.ensure_access(va, size_of::<T>()*self.len, PageFaultAccessType::WRITE).ok()?;
        Some(UserSlice { raw: self, _mark: PhantomData, _sum_guard: SumGuard::new(), locker: UserVmPagesLocker {  }  })
    }
}

impl<T, P: UserPtrPerm> Deref for UserSlice<T, P>  {
    type Target = UserSliceRaw<T>;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<'a, T, P: UserPtrPerm> DerefMut for UserSlice<T, P>  {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

unsafe impl<T> Send for UserSliceRaw<T> {}

#[derive(Clone)]
pub struct UserSlice<T, P: UserPtrPerm> {
    pub raw: UserSliceRaw<T>,
    _mark: PhantomData<P>,
    _sum_guard: SumGuard,
    locker: UserVmPagesLocker
}

impl<T, P: UserPtrPerm> Debug for UserSlice<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UserSlice").field("raw", &self.raw).finish()
    }
}

impl<T, P: UserPtrPerm> UserSlice<T, P> {
    pub unsafe fn cast_perm<P2: UserPtrPerm>(self) -> UserSlice<T, P2> {
        UserSlice {
            raw: self.raw,
            _mark: PhantomData,
            _sum_guard: self._sum_guard,
            locker: self.locker
        }
    }
}

impl<T, P: UserPtrRead> UserSlice<T, P> {
    pub fn to_ref<'a>(&'a self) -> &'a [T] {
        unsafe { self.raw.to_ref_unchecked() }
    }
}

impl<P: UserPtrRead> UserSlice<u8, P> {
    pub fn to_str<'a>(&'a self) -> Result<&'a str, str::Utf8Error> {
        unsafe {
            str::from_utf8(self.raw.to_ref_unchecked())
        }
    }
}


impl<T, P: UserPtrWrite> UserSlice<T, P> {
    pub fn to_mut<'a>(&'a self) -> &'a mut [T] {
        unsafe { self.raw.to_mut_unchecked() }
    }
}

unsafe impl<T, P: UserPtrPerm> Send for UserSlice<T, P> {}
