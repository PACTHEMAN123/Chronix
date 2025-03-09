
use core::{fmt::{self, Debug, Formatter}, iter::Step, ops::{Add, AddAssign, Sub, SubAssign}};

use core::ops::Range;

use crate::{config::{PAGE_SIZE, PAGE_SIZE_BITS}, mm::{page_table::PageLevel, PageTable}};

use super::{KernAddr, KernPageNum, VA_WIDTH_SV39, VPN_WIDTH_SV39};

/// virtual address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

/// virtual page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

#[allow(missing_docs)]
impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

#[allow(missing_docs)]
impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}


impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}
impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

impl Add<usize> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: usize) -> Self::Output {
        VirtAddr(self.0 + rhs)
    }
}


impl Add<usize> for VirtPageNum {
    type Output = VirtPageNum;

    fn add(self, rhs: usize) -> Self::Output {
        VirtPageNum(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}


impl AddAssign<usize> for VirtPageNum {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}


impl Sub<usize> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        VirtAddr(self.0 + rhs)
    }
}


impl Sub<usize> for VirtPageNum {
    type Output = VirtPageNum;

    fn sub(self, rhs: usize) -> Self::Output {
        VirtPageNum(self.0 + rhs)
    }
}

impl SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}


impl SubAssign<usize> for VirtPageNum {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}


impl Step for VirtAddr {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        usize::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        usize::forward_checked(start.0, count).map(|e| Self(e))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        usize::backward_checked(start.0, count).map(|e| Self(e))
    }
} 


impl Step for VirtPageNum {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        usize::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        usize::forward_checked(start.0, count).map(|e| Self(e))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        usize::backward_checked(start.0, count).map(|e| Self(e))
    }
} 