//! ext4 file system implementation for VFS file system type

use crate::devices::BlockDevice;
use crate::fs::vfs::{
    fstype::{FSType, FSTypeInner},
    dentry::{Dentry, DentryState, DCACHE},
    fstype::MountFlags,
    SuperBlockInner,
    inode::{Inode, InodeInner},
};
use crate::fs::SuperBlock;

use alloc::string::ToString;
use lwext4_rust::InodeTypes;
use alloc::sync::Arc;

use super::{
    Ext4Dentry, Ext4Inode, Ext4SuperBlock
};

pub struct Ext4FSType {
    inner: FSTypeInner,
}

impl Ext4FSType {
    pub fn new() -> Arc<Self> {
        Arc::new(Self{
            inner: FSTypeInner::new("ext4"),
        })
    }
}

impl FSType for Ext4FSType {
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
        let sb = Ext4SuperBlock::new(SuperBlockInner::new(dev, fs_type.clone()));
        let root_inode = Arc::new(Ext4Inode::new(Arc::downgrade(&sb), "/", InodeTypes::EXT4_DE_DIR)); 
        let root_dentry = Ext4Dentry::new(name, parent.clone());
        root_dentry.set_inode(root_inode);
        root_dentry.set_state(DentryState::USED);
        sb.set_root_dentry(root_dentry.clone());
        DCACHE.lock().insert("/".to_string(), root_dentry.clone());
        self.add_sb(&root_dentry.path(), sb);
        Some(root_dentry)
    }
}