use core::{iter::Step, ops::{Add, AddAssign, Sub, SubAssign}};

use alloc::slice;

use crate::component::constant::{Constant, ConstantsHal};

macro_rules! ImplFor {
    ($tp: tt) => {
        impl Add<usize> for $tp {
            type Output = Self;
        
            fn add(self, rhs: usize) -> Self::Output {
                Self::from(self.0 + rhs)
            }
        }
        impl Sub<usize> for $tp {
            type Output = Self;
        
            fn sub(self, rhs: usize) -> Self::Output {
                Self::from(self.0 - rhs)
            }
        }
        impl Step for $tp {
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                usize::steps_between(&start.0, &end.0)
            }
        
            fn forward_checked(start: Self, count: usize) -> Option<Self> {
                usize::forward_checked(start.0, count).map(|i| Self(i))
            }
        
            fn backward_checked(start: Self, count: usize) -> Option<Self> {
                usize::backward_checked(start.0, count).map(|i| Self(i))
            }
        }
        impl AddAssign<usize> for $tp {
            fn add_assign(&mut self, rhs: usize) {
                *self = Self::from(self.0 + rhs)
            }
        }
        impl SubAssign<usize> for $tp {
            fn sub_assign(&mut self, rhs: usize) {
                *self = Self::from(self.0 - rhs)
            }
        }
        
    };
}


#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

ImplFor!(PhysAddr);

impl alloc::fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysAddr: {:#x}", self.0)
    }
}

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << Constant::PA_WIDTH) - 1))
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

impl alloc::fmt::Debug for PhysPageNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysPageNum: {:#x}", self.0)
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value & ((1 << Constant::PPN_WIDTH) - 1))
    }
}

ImplFor!(PhysPageNum);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

impl alloc::fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtAddr: {:#x}", self.0)
    }
}

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        if value & (1usize << (Constant::VA_WIDTH-1)) == 0 {
            Self(value & ((1usize << Constant::VA_WIDTH) - 1))
        } else {
            Self(value | !((1usize << Constant::VA_WIDTH) - 1))
        }
    }
}

impl VirtAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & ((1usize << Constant::PAGE_SIZE_BITS) - 1)
    }
}

ImplFor!(VirtAddr);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

impl alloc::fmt::Debug for VirtPageNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtPageNum: {:#x}", self.0)
    }
}


impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self {
        Self(value & ((1 << Constant::VPN_WIDTH) - 1))
    }
}
ImplFor!(VirtPageNum);

pub trait VirtAddrHal {
    fn floor(&self) -> VirtPageNum;
    fn ceil(&self) -> VirtPageNum;
}

pub trait VirtPageNumHal {
    fn indexes(&self) -> [usize; Constant::PG_LEVEL];
    fn start_addr(&self) -> VirtAddr;
    fn end_addr(&self) -> VirtAddr;
}

pub trait PhysAddrHal {
    fn get_ptr<T>(&self) -> *mut T;

    fn get_mut<T>(&self) -> &'static mut T {
        unsafe { &mut *self.get_ptr() }
    }

    fn get_ref<T>(&self) -> &'static T {
        unsafe { & *self.get_ptr() }
    }

    fn get_slice<T>(&self, len: usize) -> &'static [T] {
        unsafe {
            slice::from_raw_parts(self.get_ptr(), len)
        }
    }

    fn get_slice_mut<T>(&self, len: usize) -> &'static mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.get_ptr(), len)
        }
    }

    fn floor(&self) -> PhysPageNum;

    fn ceil(&self) -> PhysPageNum;
}

pub trait PhysPageNumHal {
    fn start_addr(&self) -> PhysAddr;
    fn end_addr(&self) -> PhysAddr;
}

pub trait RangePPNHal {
    fn get_slice<T>(&self) -> &[T];
    fn get_slice_mut<T>(&self) -> &mut [T];
}