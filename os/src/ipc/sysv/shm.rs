use async_trait::async_trait;
use alloc::{borrow::ToOwned, boxed::Box, collections::btree_map::BTreeMap, sync::{Arc, Weak}, vec::Vec};
use hal::{addr::RangePPNHal, constant::{Constant, ConstantsHal}, println};
use crate::{fs::{page::{cache::PageCache, page::Page}, vfs::{File, FileInner, Inode}}, mm::allocator::{FrameAllocator, SlabAllocator}, sync::mutex::SpinNoIrqLock, syscall::SysError, task::{TidAllocator, TidHandle}};

/// shared memory manager instance
pub static SHM_MANAGER: ShmManager = ShmManager::new();

/// shared memory object
pub struct ShmObj {
    id: usize,
    cache: PageCache,
}

unsafe impl Send for ShmObj {}
unsafe impl Sync for ShmObj {}

impl ShmObj {
    /// new
    pub fn new(_size: usize) -> Arc<Self> {
        let id = SHM_MANAGER.id_alloc.lock().alloc();
        let ret = Arc::new(
                Self {
                id,
                cache: PageCache::new()
            }
        );
        SHM_MANAGER.files.lock().insert(ret.id, Arc::downgrade(&ret));
        ret
    }
}

impl Drop for ShmObj {
    fn drop(&mut self) {
        SHM_MANAGER.remove(self.id);
    }
}

impl ShmObj {
    /// get id
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// read_page_at
    pub fn read_page_at(self: Arc<Self>, offset: usize) -> Option<Arc<Page>> {
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

/// shared memory manager
pub struct ShmManager {
    files: SpinNoIrqLock<BTreeMap<usize, Weak<ShmObj>>>,
    id_alloc: SpinNoIrqLock<ShmIdAllocator>
}

impl ShmManager {
    /// 
    const fn new() -> Self {
        Self {
            files: SpinNoIrqLock::new(BTreeMap::new()),
            id_alloc: SpinNoIrqLock::new(ShmIdAllocator::new())
        }
    }
    /// 
    pub fn get(&self, id: usize) -> Option<Arc<ShmObj>> {
        if let Some(file) = self.files.lock().get(&id) {
            file.upgrade()
        } else {
            None
        }
    }
    ///
    pub fn remove(&self, id: usize) -> Option<Weak<ShmObj>> {
        self.id_alloc.lock().dealloc(id);
        self.files.lock().remove(&id)
    }
}

///Shm Id Allocator struct
pub struct ShmIdAllocator {
    current: usize,
    recycled: Vec<usize, SlabAllocator>,
}

impl ShmIdAllocator {
    ///Create an empty `TidAllocator`
    pub const fn new() -> Self {
        Self {
            current: 1,
            recycled: Vec::new_in(SlabAllocator),
        }
    }
    ///Allocate a tid
    pub fn alloc(&mut self) -> usize {
        if let Some(tid) = self.recycled.pop() {
            tid
        } else {
            self.current += 1;
            self.current - 1
        }
    }
    ///Recycle a id
    pub fn dealloc(&mut self, id: usize) {
        assert!(id < self.current);
        if id == 0 {
            return;
        }
        assert!(
            !self.recycled.iter().any(|pid| *pid == id),
            "pid {} has been deallocated!",
            id
        );
        self.recycled.push(id);
    }
}

/// 
pub fn get_shm(id: usize) -> Option<Arc<ShmObj>> {
    if id == 0 {
        return None;
    }
    SHM_MANAGER.get(id)
}
