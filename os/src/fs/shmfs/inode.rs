use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use hal::{addr::{PhysPageNum, RangePPNHal, VirtPageNum}, constant::{Constant, ConstantsHal}, util::smart_point::StrongArc};

use crate::{fs::{page::{cache::PageCache, page::Page}, vfs::{File, Inode, InodeInner}, StatxTimestamp, Xstat, XstatMask}, mm::{allocator::{FrameAllocator, SlabAllocator}, FrameTracker}, sync::mutex::SpinNoIrqLock};

/// Shared Memory Inode
#[allow(missing_docs, unused)]
pub struct ShmInode {
    pub size: AtomicUsize,
    pub cache: Arc<PageCache>,
}

impl ShmInode {
    pub fn new(size: usize) -> Self {
        Self {
            size: AtomicUsize::new(size),
            cache: Arc::new(PageCache::new())
        }
    }
}

impl Inode for ShmInode {

    fn read_page_at(self: Arc<Self>, offset: usize) -> Option<Arc<crate::fs::page::page::Page>> {
        if offset % Constant::PAGE_SIZE != 0 {
            return None;
        }
        if let Some(page) = self.cache.get_page(offset) {
            Some(page)
        } else {
            let page = Page::new(offset);
            page.frame().range_ppn.get_slice_mut::<usize>().fill(0);
            self.cache.insert_page(offset, page.clone());
            Some(page)
        }
    }
}
