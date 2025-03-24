//! fat32 file system implementation for VFS file system type

use alloc::{string::{String, ToString}, sync::Arc};

use crate::{devices::BlockDevice, fs::{vfs::{fstype::{FSType, FSTypeInner, MountFlags}, inode::{InodeInner, InodeMode}, Dentry, DentryState, DCACHE}, SuperBlock, SuperBlockInner}, sync::UPSafeCell};

use super::{dentry::FatDentry, inode::{FatDirInode, FatDirMeta}, superblock::FatSuperBlock};


pub struct Fat32FSType {
    inner: FSTypeInner,
}

impl Fat32FSType {
    #[allow(unused)]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: FSTypeInner::new("fat32"),
        })
    }
}

impl FSType for Fat32FSType {
    fn inner(&self) -> &FSTypeInner {
        &self.inner
    }
    fn kill_sb(&self) -> isize {
        todo!()
    }
    fn mount(&'static self, name: &str, parent: Option<Arc<dyn Dentry>>, _flags: MountFlags, dev: Option<Arc<dyn BlockDevice>>) -> Option<Arc<dyn Dentry>> {
        // can be dangerous..
        let fs_type = unsafe {
            let ptr: *const dyn FSType = self;
            Arc::from_raw(ptr)
        };
        let sb = FatSuperBlock::new(SuperBlockInner::new(dev, fs_type.clone()));
        self.add_sb(name, sb);
        let sb = self.get_static_sb(name);
        let dir = sb.get_root_inode(name);
        let root_dentry = FatDentry::new(name, sb.clone(), parent.clone());
        root_dentry.set_inode(dir);
        root_dentry.set_state(DentryState::USED);
        sb.set_root_dentry(root_dentry.clone());
        DCACHE.lock().insert("/".to_string(), root_dentry.clone());
        Some(root_dentry)
    }
}