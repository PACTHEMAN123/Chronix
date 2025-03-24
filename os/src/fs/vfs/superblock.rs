//! vfs super block
//! 
use alloc::sync::{Arc, Weak};
use spin::Once;

use crate::devices::BlockDevice;
use crate::fs::vfs::Inode;

use super::fstype::FSType;
use super::Dentry;

/// the base of super block of all file system
pub struct SuperBlockInner {
    /// the block device fs using
    pub device: Option<Arc<dyn BlockDevice>>,
    /// file system type
    pub fs_type: Weak<dyn FSType>,
    /// the root dentry to the mount point
    pub root: Once<Arc<dyn Dentry>>,
}

impl SuperBlockInner {
    /// create a super block inner with device
    pub fn new(device: Option<Arc<dyn BlockDevice>>, fs_type: Arc<dyn FSType>) -> Self {
        Self {
            device,
            fs_type: Arc::downgrade(&fs_type),
            root: Once::new(),
        }
    }
}

/// super block trait left for file system implement
pub trait SuperBlock: Send + Sync {
    /// get the inner data of superblock
    fn inner(&self) -> &SuperBlockInner;
    /// set root
    fn set_root_dentry(&self, root: Arc<dyn Dentry>) {
        self.inner().root.call_once(|| root);
    }
    /// get root dir inode (will only use construct)
    fn get_root_inode(&'static self, name: &str) -> Arc<dyn Inode>;
}

impl dyn SuperBlock {
    /// get the root dentry
    pub fn root(&self) -> Arc<dyn Dentry> {
        self.inner().root.get().unwrap().clone()
    }
}