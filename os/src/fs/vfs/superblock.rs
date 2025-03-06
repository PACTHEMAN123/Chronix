//! vfs super block
//! 
use alloc::sync::Arc;

use crate::devices::BlockDevice;
use crate::fs::ext4::Inode;

/// the base of super block of all file system
pub struct SuperBlockInner {
    /// the block device fs using
    pub device: Option<Arc<dyn BlockDevice>>,
    /// the root inode
    pub root: Option<Arc<Inode>>,
}

impl SuperBlockInner {
    /// create a super block inner with device
    pub fn new(device: Option<Arc<dyn BlockDevice>>, root: Option<Arc<Inode>>) -> Self {
        Self {
            device,
            root,
        }
    }
}

/// super block trait left for file system implement
pub trait SuperBlock: Send + Sync {
    /// get the inner data of superblock
    fn inner(&self) -> &SuperBlockInner;
}

impl dyn SuperBlock {
    /// get the root inode
    pub fn root(&self) -> Arc<Inode> {
        Arc::clone(&self.inner().root.as_ref().unwrap())
    }
}