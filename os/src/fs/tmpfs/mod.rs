//! Tmp file system

use alloc::sync::Arc;

use super::vfs::Dentry;


pub mod superblock;
pub mod fstype;
pub mod inode;
pub mod dentry;
pub mod file;

/// init the /tmp
pub fn init_tmpfs(_root_dentry: Arc<dyn Dentry>) {
    // do nothing
}