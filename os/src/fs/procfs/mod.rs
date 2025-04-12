//! proc file system

use alloc::sync::Arc;
use meminfo::{MemInfoDentry, MemInfoInode};
use mounts::{MountsDentry, MountsInode};
use self_::{ExeDentry, ExeInode};

use super::{simplefs::{dentry::SpDentry, inode::SpInode}, vfs::{Dentry, DCACHE}};

pub mod fstype;
pub mod superblock;
pub mod self_;
pub mod mounts;
pub mod meminfo;

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

    // touch /proc/meminfo
    let mem_dentry = MemInfoDentry::new("meminfo", sb.clone(), Some(root_dentry.clone()));
    let mem_inode = MemInfoInode::new(sb.clone());
    mem_dentry.set_inode(mem_inode);
    root_dentry.add_child(mem_dentry.clone());
    DCACHE.lock().insert(mem_dentry.path(), mem_dentry.clone());

    // touch /proc/mounts
    let mounts_dentry = MountsDentry::new("mounts", sb.clone(), Some(root_dentry.clone()));
    let mounts_inode = MountsInode::new(sb.clone());
    mounts_dentry.set_inode(mounts_inode);
    root_dentry.add_child(mounts_dentry.clone());
    DCACHE.lock().insert(mounts_dentry.path(), mounts_dentry.clone());

}
