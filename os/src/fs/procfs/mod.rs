//! proc file system

use alloc::sync::Arc;
use self_::{ExeDentry, ExeInode};

use super::{simplefs::{dentry::SpDentry, inode::SpInode}, vfs::{Dentry, DCACHE}};

pub mod fstype;
pub mod superblock;
pub mod self_;

/// init the whole /proc
pub fn init_procfs(root_dentry: Arc<dyn Dentry>) {
    let sb = root_dentry.superblock();

    // mkdir /proc/self
    let self_dentry = SpDentry::new("self", sb.clone(), Some(root_dentry.clone()));
    let self_inode = SpInode::new(sb.clone());
    self_dentry.set_inode(self_inode);
    root_dentry.add_child(self_dentry.clone());
    DCACHE.lock().insert(self_dentry.path(), self_dentry.clone());

    // touch /proc/self/exe
    let exe_dentry = ExeDentry::new(sb.clone(), Some(root_dentry.clone()));
    let exe_inode = ExeInode::new(sb.clone());
    exe_dentry.set_inode(exe_inode);
    self_dentry.add_child(exe_dentry.clone());
    DCACHE.lock().insert(exe_dentry.path(), exe_dentry.clone());

}
