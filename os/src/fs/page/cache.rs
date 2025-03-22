//! Page Cache
//! each inode will hold a page cache
//! (todos): 1. radix tree to manage the offset to page
//! 2. ahead read 

use core::{cmp, sync::atomic::{AtomicUsize, Ordering}};

use crate::{fs::vfs::Inode, sync::mutex::SpinNoIrqLock};
use alloc::sync::Arc;
use hashbrown::HashMap;
use log::info;

use super::page::{Page, PAGE_SIZE};

pub struct PageCache {
    /// from file offset(should be page aligned)
    /// to the cached page
    pages: SpinNoIrqLock<HashMap<usize, Arc<Page>>>,
    /// the postion of EOF
    /// save it to prevent endless read
    /// notice that it may need to update when 
    /// cache write, as it may lead to expand the file
    end: AtomicUsize,
}

impl PageCache {
    /// create a new Page Cache
    pub fn new() -> Self {
        Self {
            pages: SpinNoIrqLock::new(HashMap::new()),
            end: AtomicUsize::new(0usize),
        }
    }
    /// get the cache inner
    pub fn get_pages(&self) -> &SpinNoIrqLock<HashMap<usize, Arc<Page>>> {
        &self.pages
    }
    /// get the page at file offset
    pub fn get_page(&self, offset: usize) -> Option<Arc<Page>> {
        assert!(offset % PAGE_SIZE == 0);
        self.pages.lock().get(&offset).cloned()
    }
    /// insert the page at file offset
    pub fn insert_page(&self, offset: usize, page: Arc<Page>) {
        assert!(offset % PAGE_SIZE == 0);
        self.pages.lock().insert(offset, page);
    }
    pub fn update_end(&self, offset: usize) {
        let end = self.end.load(Ordering::Acquire);
        self.end.store(cmp::max(end, offset), Ordering::Release);
    }
    pub fn end(&self) -> usize {
        self.end.load(Ordering::Acquire)
    }
    /// flush all dirty pages
    pub fn flush(&self, inode: Arc<dyn Inode>) {
        info!("start to flush all pages");
        let mut pages = self.pages.lock();
        for (&offset, page) in pages.iter_mut() {
            if page.is_dirty() == false {
                continue;
            }
            inode.write_at(offset, page.get_slice::<u8>()).expect("[PageCache]: failed at flush");
        }
    }
}