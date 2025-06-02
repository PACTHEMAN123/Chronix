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
    pub fn new(inner: SuperBlockInner, mount_point: &'static str, device_name: &'static str) -> Arc<dyn SuperBlock> {
        log::info!("mount a ext fs at {}, device name {}", mount_point, device_name);
        let block_device = inner.device.as_ref().unwrap().clone();
        let disk = Disk::new(block_device);
        let block = Ext4BlockWrapper::<Disk>::new(disk, mount_point, device_name).expect("failed to create ext4fs");
        Arc::new(Self {inner, block})
    }
}

impl SuperBlock for Ext4SuperBlock {
    fn inner(&self) -> &SuperBlockInner {
        &self.inner
    }
    fn get_root_inode(&'static self, _name: &str) -> Arc<dyn Inode> {
        self.inner().root.get().unwrap().clone().inode().unwrap()
    }
}
