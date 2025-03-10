//! ext4 file system implement for the VFS super block
use crate::fs::vfs::{Dentry, DentryInner, DentryState, Inode, SuperBlock, SuperBlockInner, DCACHE};
use alloc::string::ToString;
use lwext4_rust::{Ext4BlockWrapper, Ext4File, InodeTypes, KernelDevOp};
use super::{disk::Disk, Ext4Dentry};
use super::inode::Ext4Inode;
use alloc::sync::{Arc, Weak};

#[allow(dead_code)]
/// EXT4 FS super block
pub struct Ext4SuperBlock {
    /// basic data
    inner: SuperBlockInner,
    /// lwext4 object to control file system
    block: Ext4BlockWrapper<Disk>,
}

unsafe impl Send for Ext4SuperBlock {}
unsafe impl Sync for Ext4SuperBlock {}

// EXT4 FS super block implement 
impl Ext4SuperBlock {
    /// create a new ext4 super block using device
    pub fn new(inner: SuperBlockInner) -> Arc<Self> {
        let block_device = inner.device.as_ref().unwrap().clone();
        let disk = Disk::new(block_device);
        let block = Ext4BlockWrapper::<Disk>::new(disk).expect("failed to create ext4fs");
        let super_block = Arc::new(Self {inner, block});
        
        // need to reset the super block's root
        // now we simply mount the ext4fs at "/"
        let root = Arc::new(Ext4Inode::new(super_block.clone(), "/", InodeTypes::EXT4_DE_DIR));
        super_block.set_root(root.clone());

        // create root dentry and insert into dache
        let root_dentry = Ext4Dentry::new("/", super_block.clone(), None);
        root_dentry.set_inode(root.clone());
        root_dentry.set_state(DentryState::USED);
        DCACHE.lock().insert("/".to_string(), root_dentry);

        Arc::clone(&super_block)
    }
}

impl SuperBlock for Ext4SuperBlock {
    fn inner(&self) -> &SuperBlockInner {
        &self.inner
    }
}
