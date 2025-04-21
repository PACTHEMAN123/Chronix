use async_trait::async_trait;
use alloc::{borrow::ToOwned, boxed::Box, collections::btree_map::BTreeMap, sync::{Arc, Weak}, vec::Vec};
use hal::println;
use crate::{fs::vfs::{File, FileInner, Inode}, mm::allocator::{FrameAllocator, SlabAllocator}, sync::mutex::SpinNoIrqLock, syscall::SysError, task::{TidAllocator, TidHandle}};

use super::inode::ShmInode;

pub static SHM_MANAGER: ShmManager = ShmManager::new();

pub struct ShmInodeWrapper(SpinNoIrqLock<Arc<dyn Inode>>);

unsafe impl Send for ShmInodeWrapper {}
unsafe impl Sync for ShmInodeWrapper {}

pub struct ShmFile {
    id: usize,
    inode: ShmInodeWrapper,
}

unsafe impl Send for ShmFile {}
unsafe impl Sync for ShmFile {}

impl ShmFile {
    pub fn new(size: usize) -> Arc<Self> {
        let id = SHM_MANAGER.id_alloc.lock().alloc();
        let ret = Arc::new(
                Self {
                id,
                inode: ShmInodeWrapper(SpinNoIrqLock::new(Arc::new(ShmInode::new(size))))
            }
        );
        SHM_MANAGER.files.lock().insert(ret.id, Arc::downgrade(&ret));
        ret
    }
}

impl Drop for ShmFile {
    fn drop(&mut self) {
        SHM_MANAGER.files.lock().remove(&self.id);
        SHM_MANAGER.id_alloc.lock().dealloc(self.id);
    }
}

#[async_trait]
impl File for ShmFile {
    fn file_inner(&self) -> &FileInner {
        panic!("shmfile don't have inner")
    }
    fn inode(&self) -> Option<Arc<dyn Inode>> {
        Some(self.inode.0.lock().clone())
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    async fn read(&self, _buf: &mut [u8]) -> Result<usize, SysError> {
        panic!("cannot read sp file")
    }
    async fn write(&self, _buf: &[u8]) -> Result<usize, SysError> {
        panic!("cannot write sp file")
    }
}

pub struct ShmManager {
    files: SpinNoIrqLock<BTreeMap<usize, Weak<ShmFile>>>,
    id_alloc: SpinNoIrqLock<ShmIdAllocator>
}

impl ShmManager {
    const fn new() -> Self {
        Self {
            files: SpinNoIrqLock::new(BTreeMap::new()),
            id_alloc: SpinNoIrqLock::new(ShmIdAllocator::new())
        }
    }

    pub fn get(&self, id: usize) -> Option<Arc<ShmFile>> {
        if let Some(file) = self.files.lock().get(&id) {
            file.upgrade()
        } else {
            None
        }
    }

    pub fn remove(&self, id: usize) -> Option<Weak<ShmFile>> {
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
            current: 0,
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
        assert!(
            !self.recycled.iter().any(|pid| *pid == id),
            "pid {} has been deallocated!",
            id
        );
        self.recycled.push(id);
    }
}
