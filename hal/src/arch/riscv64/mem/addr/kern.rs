use crate::hal::mem::{KernAddr, KernAddrHal, KernPageNum, KernPageNumHal, PageNumber, PageNumberHal};


impl KernAddrHal for KernAddr {
    fn get_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl KernPageNumHal for KernPageNum {

    type PageNumType = PageNumber;

    fn get_ptr<T>(&self) -> *mut T {
        (self.0 << Self::PageNumType::PAGE_SIZE_BITS) as *mut T
    }

}
