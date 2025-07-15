use alloc::sync::Arc;

use crate::{devices::BlockDevice, fs::{vfs::{fstype::{FSType, FSTypeInner, MountFlags}, inode::InodeMode, Dentry, DentryState, DCACHE}, SuperBlock, SuperBlockInner}};

use super::{dentry::TmpDentry, inode::TmpInode, superblock::TmpSuperBlock};


pub struct TmpFSType {
    inner: FSTypeInner,
}

impl TmpFSType {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: FSTypeInner::new("tmpfs"),
        })
    }
}

impl FSType for TmpFSType {
    fn inner(&self) -> &FSTypeInner {
        &self.inner
    }

    fn mount(&'static self, name: &str, parent: Option<Arc<dyn Dentry>>, _flags: MountFlags, dev: Option<Arc<dyn BlockDevice>>) -> Option<Arc<dyn Dentry>> {
        let fs_type = unsafe {
            let ptr: *const dyn FSType = self;
            Arc::from_raw(ptr)
        };
        let sb = TmpSuperBlock::new(SuperBlockInner::new(dev, fs_type.clone()));
        let root_inode = TmpInode::new(Arc::downgrade(&sb), InodeMode::DIR);
        let root_dentry = TmpDentry::new(name, parent.clone());
        root_dentry.set_inode(root_inode);
        root_dentry.set_state(DentryState::USED);
        sb.set_root_dentry(root_dentry.clone());
        DCACHE.lock().insert(root_dentry.path(), root_dentry.clone());
        self.add_sb(&root_dentry.path(), sb);
        Some(root_dentry)
    }

    fn kill_sb(&self) -> isize {
        todo!()
    }
}