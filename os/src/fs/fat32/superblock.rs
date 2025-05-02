//! fat32 file system implement for the VFS super block

use crate::{fs::{vfs::{inode::InodeMode, Inode, InodeInner}, SuperBlock, SuperBlockInner}, sync::UPSafeCell};
use alloc::{string::String, sync::Arc};
use fatfs::{Dir, Error, File, LossyOemCpConverter, NullTimeProvider};

use super::{disk::DiskCursor, inode::{FatDirInode, FatDirMeta}};


pub struct FatSuperBlock {
    /// basic data
    pub(crate) inner: SuperBlockInner,
    /// fat32 object to control filesystem
    pub(crate) block: Arc<fatfs::FileSystem<DiskCursor, NullTimeProvider, LossyOemCpConverter>>,
}

unsafe impl Send for FatSuperBlock {}
unsafe impl Sync for FatSuperBlock {}

// FAT32 FS super block implement
impl FatSuperBlock {
    /// create a new fat32 super block using device
    pub fn new(inner: SuperBlockInner) -> Arc<Self> {
        let block_device = inner.device.as_ref().unwrap().clone();
        let cursor = DiskCursor::new(block_device);
        let block = Arc::new(fatfs::FileSystem::new(cursor, fatfs::FsOptions::new()).expect("open fs wrong"));
        Arc::new(Self {inner, block })
    }
}

impl SuperBlock for FatSuperBlock {
    fn inner(&self) -> &SuperBlockInner {
        &self.inner
    }
    fn get_root_inode(&'static self, name: &str) -> Arc<dyn Inode> {
        let sb = unsafe {
            let ptr: *const dyn SuperBlock = self;
            Arc::from_raw(ptr)
        };
        let dir = Arc::new(FatDirInode {
            inner: InodeInner::new(
            Some(Arc::downgrade(&sb)),
            InodeMode::DIR,
            0,
            ),
            dir: UPSafeCell::new(FatDirMeta {
                name: String::from(name),
                inner: self.block.root_dir(),
                size: 0,
            }),
        });
        dir
    }
}
