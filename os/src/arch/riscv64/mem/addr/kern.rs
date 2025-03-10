use hal::mem::{KernAddrHal, KernPageNumHal, PageNumberHal};

use super::PageNum;

pub struct KernAddr(pub usize);

impl KernAddrHal for KernAddr {
    fn get_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

pub struct KernPageNum(pub usize);

impl KernPageNumHal for KernPageNum {

    type PageNumType = PageNum;

    fn get_ptr<T>(&self) -> *mut T {
        (self.0 << Self::PageNumType::PAGE_SIZE_BITS) as *mut T
    }

}
