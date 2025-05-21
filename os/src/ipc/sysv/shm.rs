use async_trait::async_trait;
use alloc::{borrow::ToOwned, boxed::Box, collections::{btree_map::BTreeMap, btree_set::BTreeSet}, sync::{Arc, Weak}, vec::Vec};
use hal::{addr::RangePPNHal, constant::{Constant, ConstantsHal}, println};
use crate::{fs::{page::{cache::PageCache, page::Page}, vfs::{File, FileInner, Inode}}, mm::allocator::{FrameAllocator, SlabAllocator}, sync::mutex::SpinNoIrqLock, syscall::SysError, task::{TidAllocator, TidHandle}, timer::get_current_time_sec};

use super::IpcPerm;

/// shared memory manager instance
pub static SHM_MANAGER: ShmManager = ShmManager::new();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShmIdDs {
    // Ownership and permissions
    pub perm: IpcPerm,
    // Size of segment (bytes). In our system, this must be aligned
    pub segsz: usize,
    // Last attach time
    pub atime: usize,
    // Last detach time
    pub dtime: usize,
    // Creation time/time of last modification via shmctl()
    pub ctime: usize,
    // PID of creator
    pub cpid: usize,
    // PID of last shmat(2)/shmdt(2)
    pub lpid: usize,
    // No. of current attaches
    pub nattch: usize,
}

impl ShmIdDs {
    pub fn new(sz: usize, cpid: usize) -> Self {
        Self {
            perm: IpcPerm::default(),
            segsz: sz,
            atime: 0,
            dtime: 0,
            ctime: get_current_time_sec(),
            cpid: cpid,
            lpid: 0,
            nattch: 0,
        }
    }

    pub fn attach(&mut self, lpid: usize) {
        // shm_atime is set to the current time.
        self.atime = get_current_time_sec();
        // shm_lpid is set to the process-ID of the calling process.
        self.lpid = lpid;
        // shm_nattch is incremented by one.
        self.nattch += 1;
    }

    /// return whether the SHARED_MEMORY_MANAGER should remove the SharedMemory
    /// which self ShmIdDs belongs to;
    pub fn detach(&mut self, lpid: usize) -> bool {
        // shm_dtime is set to the current time.
        self.dtime = get_current_time_sec();
        // shm_lpid is set to the process-ID of the calling process.
        self.lpid = lpid;
        // shm_nattch is decremented by one.
        self.nattch -= 1;
        if self.nattch == 0 {
            return true;
        }
        false
    }
}

/// shared memory object
pub struct ShmObj {
    id: usize,
    pub shmid_ds: SpinNoIrqLock<ShmIdDs>,
    cache: PageCache,
}

unsafe impl Send for ShmObj {}
unsafe impl Sync for ShmObj {}

impl ShmObj {
    /// new
    fn new(id:usize, size: usize, pid: usize) -> Self {
        let ret = Self {
            id,
            shmid_ds: SpinNoIrqLock::new(ShmIdDs::new(size, pid)),
            cache: PageCache::new()
        };
        ret
    }
}

impl ShmObj {
    /// get id
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// read_page_at
    pub fn read_page_at(self: Arc<Self>, offset: usize) -> Option<Arc<Page>> {
        if offset % Constant::PAGE_SIZE != 0 || offset >= self.shmid_ds.lock().segsz {
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

impl Drop for ShmObj {
    fn drop(&mut self) {
        let _ = SHM_MANAGER.id_alloc.lock().dealloc(self.id);
    }
}

/// shared memory manager
pub struct ShmManager {
    files: SpinNoIrqLock<BTreeMap<usize, Arc<ShmObj>>>,
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
        if id == 0 {
            return None;
        }
        if let Some(file) = self.files.lock().get(&id).cloned() {
            Some(file)
        } else {
            None
        }
    }
    pub fn alloc(&self, size: usize, pid: usize) -> Option<Arc<ShmObj>> {
        let id = self.id_alloc.lock().alloc()?;
        let shm = Arc::new(ShmObj::new(id, size, pid));
        self.files.lock().insert(id, shm.clone());
        Some(shm)
    }
    pub fn alloc_at(&self, size: usize, pid: usize, id: usize) -> Option<Arc<ShmObj>> {
        if id == 0 {
            return self.alloc(size, pid);
        }
        if let Some(file) = self.files.lock().get(&id).cloned() {
            Some(file)
        } else {
            let shm = Arc::new(ShmObj::new(id, size, pid));
            self.files.lock().insert(id, shm.clone());
            Some(shm)
        }
    }
    ///
    pub fn remove(&self, id: usize) -> Option<Arc<ShmObj>> {
        self.files.lock().remove(&id)
    }
}

///Shm Id Allocator struct
pub struct ShmIdAllocator {
    cur: usize,
    pool: BTreeSet<usize>
}

impl ShmIdAllocator {
    ///Create an empty `TidAllocator`
    pub const fn new() -> Self {
        Self {
            cur: 0,
            pool: BTreeSet::new()
        }
    }
    ///Allocate a id
    pub fn alloc(&mut self) -> Option<usize> {
        let mut times = 1000;
        loop {
            if self.cur == usize::MAX {
                self.cur = 0;
            }
            self.cur += 1;
            if self.pool.insert(self.cur) {
                break Some(self.cur)
            }
            times -= 1;
            if times == 0 {
                break None;
            }
        }
    }
    /// Allocate at a given id
    /// If id is equal to 0, this function functions the same as function alloc
    pub fn alloc_at(&mut self, id: usize) -> Option<usize> {
        if id == 0 {
            self.alloc()
        } else if self.pool.contains(&id) {
            None
        } else {
            self.cur = self.cur.max(id);
            self.pool.insert(id);
            Some(id)
        }
    }
    ///Recycle a id
    pub fn dealloc(&mut self, id: usize) -> Result<(), ()> {
        if self.pool.remove(&id) {
            Ok(())
        } else {
            Err(())
        }
    }
}
