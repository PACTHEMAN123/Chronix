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


