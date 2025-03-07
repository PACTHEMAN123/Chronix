//! vfs super block
//! 
use alloc::sync::Arc;
use spin::Once;

use crate::devices::BlockDevice;
use crate::fs::vfs::Inode;

/// the base of super block of all file system
pub struct SuperBlockInner {
    /// the block device fs using
    pub device: Option<Arc<dyn BlockDevice>>,
    /// the root inode
    pub root: Once<Arc<dyn Inode>>,
}

impl SuperBlockInner {
    /// create a super block inner with device
    pub fn new(device: Option<Arc<dyn BlockDevice>>) -> Self {
        Self {
            device,
            root: Once::new(),
        }
    }
}

/// super block trait left for file system implement
pub trait SuperBlock: Send + Sync {
    /// get the inner data of superblock
    fn inner(&self) -> &SuperBlockInner;
    /// set root
    fn set_root(&self, root: Arc<dyn Inode>) {
        self.inner().root.call_once(|| root);
    }
}

impl dyn SuperBlock {
    /// get the root inode
    pub fn root(&self) -> Arc<dyn Inode> {
        Arc::clone(&self.inner().root.get().unwrap())
    }
}