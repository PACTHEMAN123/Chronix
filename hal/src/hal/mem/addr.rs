use core::{iter::Step, ops::{Add, AddAssign, Sub, SubAssign}, usize};

const fn bits(x: usize) -> usize {
    let mut i = 63;
    loop {
        if x & (1 << i) != 0 {
            break i + 1
        }
        if i == 0 {
            break 0
        }
        i -= 1;
    }
}

#[allow(unused, missing_docs)]
pub trait PageNumberHal {
    const PAGE_SIZE: usize;
    const PAGE_SIZE_BITS: usize = bits(Self::PAGE_SIZE);
}

#[allow(unused, missing_docs)]
pub trait VirtAddrHal 
    : Clone + Copy
    + Step + Add<usize> + Sub<usize>
    + PartialEq + Eq
    + PartialOrd + Ord
{
    const VA_WIDTH: usize;
    type VirtPageNum: VirtPageNumHal;

    fn floor(&self) -> Self::VirtPageNum;
    fn ceil(&self) -> Self::VirtPageNum;
}

#[allow(unused, missing_docs)]
pub trait PhysAddrHal
    : Clone + Copy
    + Step + Add<usize> + Sub<usize>
    + PartialEq + Eq
    + PartialOrd + Ord
{
    const PA_WIDTH: usize;
    type KernAddr: KernAddrHal;
    fn to_kern(&self) -> Self::KernAddr;
}

#[allow(unused, missing_docs)]
pub trait KernAddrHal
{
    fn get_ptr<T>(&self) -> *mut T;

    fn get_mut<T>(&self) -> &'static mut T {
       unsafe { &mut *self.get_ptr() }
    }

    fn get_ref<T>(&self) -> &'static T {
        unsafe { & *self.get_ptr() }
    }
}

#[allow(unused, missing_docs)]
pub trait VirtPageNumHal 
    : Clone + Copy
    + Step + Add<usize> + Sub<usize>
    + PartialEq + Eq
    + PartialOrd + Ord
{
    type AddrType: VirtAddrHal;
    type PageNumType: PageNumberHal;
    const VPN_WIDTH: usize = Self::AddrType::VA_WIDTH - Self::PageNumType::PAGE_SIZE_BITS;
    const LEVEL: usize;

    fn index(&self, i: usize) -> usize;
}

#[allow(unused, missing_docs)]
pub trait PhysPageNumHal 
    : Clone + Copy
    + Step + Add<usize> + Sub<usize>
    + PartialEq + Eq
    + PartialOrd + Ord
{
    type AddrType: PhysAddrHal;
    type PageNumType: PageNumberHal;
    const VPN_WIDTH: usize = Self::AddrType::PA_WIDTH - Self::PageNumType::PAGE_SIZE_BITS;
    type KernPageNum: KernPageNumHal;
    fn to_kern(&self) -> Self::KernPageNum;
}

#[allow(unused, missing_docs)]
pub trait KernPageNumHal
{
    type PageNumType: PageNumberHal;

    fn get_ptr<T>(&self) -> *mut T;

    fn get_mut<T>(&self) -> &'static mut T {
       unsafe { &mut *self.get_ptr() }
    }

    fn get_ref<T>(&self) -> &'static T {
        unsafe { & *self.get_ptr() }
    }
}


pub struct PageNumber;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

pub struct KernAddr(pub usize);

pub struct KernPageNum(pub usize);


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
