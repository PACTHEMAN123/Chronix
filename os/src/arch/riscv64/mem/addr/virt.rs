use core::{iter::Step, ops::{Add, Sub}};

use hal::mem::{PageNumberHal, VirtAddrHal, VirtPageNumHal};

use super::PageNum;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

impl VirtAddrHal for VirtAddr {
    const VA_WIDTH: usize = 39;
    
    type VirtPageNum = VirtPageNum;
    
    fn floor(&self) -> Self::VirtPageNum {
        VirtPageNum(self.0 >> PageNum::PAGE_SIZE_BITS)
    }
    
    fn ceil(&self) -> Self::VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 + PageNum::PAGE_SIZE - 1) >> PageNum::PAGE_SIZE_BITS)
        }
    }
}

impl Step for VirtAddr {
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

impl Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}



#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

impl VirtPageNumHal for VirtPageNum {
    type AddrType = VirtAddr;

    type PageNumType = PageNum;

    const LEVEL: usize = 3;

    fn index(&self, i: usize) -> usize {
        (self.0 >> (2 - i) * 3) & ((1 << 9) - 1)
    }
}

impl Step for VirtPageNum {
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

impl Add<usize> for VirtPageNum {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for VirtPageNum {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}