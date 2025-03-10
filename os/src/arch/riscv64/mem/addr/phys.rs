use core::{iter::Step, ops::{Add, Sub}};

use hal::mem::{PhysAddrHal, PhysPageNumHal};

use super::{PageNum, PageNumberHal, kern::KernAddr, KernPageNum, KERNEL_ADDR_OFFSET};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

impl PhysAddrHal for PhysAddr {
    const PA_WIDTH: usize = 44;

    type KernAddr = KernAddr;

    fn to_kern(&self) -> Self::KernAddr {
        KernAddr(self.0 + KERNEL_ADDR_OFFSET)
    }
}

impl Step for PhysAddr {
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

impl Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

impl PhysPageNumHal for PhysPageNum {
    type AddrType = PhysAddr;

    type PageNumType = PageNum;

    type KernPageNum = KernPageNum;

    fn to_kern(&self) -> Self::KernPageNum {
        KernPageNum(self.0 + (KERNEL_ADDR_OFFSET >> Self::PageNumType::PAGE_SIZE_BITS))
    }
}

impl Step for PhysPageNum {
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

impl Add<usize> for PhysPageNum {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for PhysPageNum {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}
