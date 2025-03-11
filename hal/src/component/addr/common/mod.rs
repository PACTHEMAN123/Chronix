use core::{iter::Step, ops::{Add, AddAssign, Sub, SubAssign}};

use crate::component::constant::{Constant, ConstantsHal};

macro_rules! ImplAddFor {
    ($tp: tt) => {
        impl Add<usize> for $tp {
            type Output = Self;
        
            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs)
            }
        }
    };
}

macro_rules! ImplSubFor {
    ($tp: tt) => {
        impl Sub<usize> for $tp {
            type Output = Self;
        
            fn sub(self, rhs: usize) -> Self::Output {
                Self(self.0 - rhs)
            }
        }
    };
}


macro_rules! ImplStepFor {
    ($tp: tt) => {
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
    };
}

macro_rules! ImplAddAssignFor {
    ($tp: tt) => {
        impl AddAssign<usize> for $tp {
            fn add_assign(&mut self, rhs: usize) {
                self.0 += rhs
            }
        }
    };
}

macro_rules! ImplSubAssignFor {
    ($tp: tt) => {
        impl SubAssign<usize> for $tp {
            fn sub_assign(&mut self, rhs: usize) {
                self.0 -= rhs
            }
        }
    };
}




#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

ImplAddFor!(PhysAddr);
ImplSubFor!(PhysAddr);
ImplStepFor!(PhysAddr);
ImplAddAssignFor!(PhysAddr);
ImplSubAssignFor!(PhysAddr);


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

ImplAddFor!(PhysPageNum);
ImplSubFor!(PhysPageNum);
ImplStepFor!(PhysPageNum);
ImplAddAssignFor!(PhysPageNum);
ImplSubAssignFor!(PhysPageNum);


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

ImplAddFor!(VirtAddr);
ImplSubFor!(VirtAddr);
ImplStepFor!(VirtAddr);
ImplAddAssignFor!(VirtAddr);
ImplSubAssignFor!(VirtAddr);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

ImplAddFor!(VirtPageNum);
ImplSubFor!(VirtPageNum);
ImplStepFor!(VirtPageNum);
ImplAddAssignFor!(VirtPageNum);
ImplSubAssignFor!(VirtPageNum);

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

    fn floor(&self) -> PhysPageNum;

    fn ceil(&self) -> PhysPageNum;
}

pub trait PhysPageNumHal {
    fn start_addr(&self) -> PhysAddr;
    fn end_addr(&self) -> PhysAddr;
}