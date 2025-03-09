mod vm_area;
mod vm_space;

use core::ops::Range;

#[allow(unused)]
pub use vm_area::{KernelVmArea, KernelVmAreaType, UserVmArea, UserVmAreaType, VmArea, VmAreaCowExt, VmAreaFrameExt, VmAreaPageFaultExt, MapPerm};
#[allow(unused)]
pub use vm_space::{KERNEL_SPACE, KernelVmSpace, UserVmSpace, VmSpace, VmSpaceHeapExt, VmSpacePageFaultExt, PageFaultAccessType, remap_test};

use super::{allocator::{frames_alloc, FrameRangeTracker}, page_table::PageLevel, VirtPageNum};

#[allow(missing_docs)]
pub struct VpnPageRangeIter {
    pub range_vpn: Range<VirtPageNum>
}

#[allow(missing_docs)]
impl VpnPageRangeIter {
    pub fn new(range_vpn: Range<VirtPageNum>) -> Self {
        Self { range_vpn }
    }
}

impl Iterator for VpnPageRangeIter {
    type Item = (VirtPageNum, PageLevel);

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_vpn.is_empty() {
            None
        } else {
            if self.range_vpn.start.0 % PageLevel::Huge.page_count() == 0 
            && self.range_vpn.clone().count() >= PageLevel::Huge.page_count() {
                let ret = (self.range_vpn.start, PageLevel::Huge);
                self.range_vpn.start += PageLevel::Huge.page_count();
                Some(ret)
            } else if self.range_vpn.start.0 % PageLevel::Big.page_count() == 0
            && self.range_vpn.clone().count() >= PageLevel::Big.page_count() {
                let ret = (self.range_vpn.start, PageLevel::Big);
                self.range_vpn.start += PageLevel::Big.page_count();
                Some(ret)
            } else {
                let ret = (self.range_vpn.start, PageLevel::Small);
                self.range_vpn.start += PageLevel::Small.page_count();
                Some(ret)
            }
        }
    }
}

#[allow(missing_docs)]
pub struct VpnPageRangeWithAllocIter {
    pub range_vpn: Range<VirtPageNum>
}

#[allow(missing_docs)]
impl VpnPageRangeWithAllocIter {
    pub fn new(range_vpn: Range<VirtPageNum>) -> Self {
        Self { range_vpn }
    }
}

impl Iterator for VpnPageRangeWithAllocIter {
    type Item = (VirtPageNum, FrameRangeTracker, PageLevel);

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_vpn.is_empty() {
            None
        } else {
            if self.range_vpn.start.0 % PageLevel::Huge.page_count() == 0 
            && self.range_vpn.clone().count() >= PageLevel::Huge.page_count() {
                if let Some(frame) = frames_alloc(PageLevel::Huge.page_count()) {
                    let ret = (self.range_vpn.start, frame, PageLevel::Huge);
                    self.range_vpn.start += PageLevel::Huge.page_count();
                    return Some(ret);
                } 
            }

            if self.range_vpn.start.0 % PageLevel::Big.page_count() == 0
            && self.range_vpn.clone().count() >= PageLevel::Big.page_count() {
                if let Some(frame) = frames_alloc(PageLevel::Big.page_count()) {
                    let ret = (self.range_vpn.start, frame, PageLevel::Big);
                    self.range_vpn.start += PageLevel::Big.page_count();
                    return Some(ret);
                } 
            }

            let frame = frames_alloc(1).unwrap();
            let ret = (self.range_vpn.start, frame, PageLevel::Small);
            self.range_vpn.start += PageLevel::Small.page_count();
            Some(ret)
            
        }
    }
}
