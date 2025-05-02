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
    let sb = root_dentry.inode().unwrap().inode_inner().super_block.clone();

    // mkdir /proc/self
    let self_dentry = SpDentry::new("self", Some(root_dentry.clone()));
    let self_inode = SpInode::new(sb.clone().unwrap());
    self_dentry.set_inode(self_inode);
    root_dentry.add_child(self_dentry.clone());
    DCACHE.lock().insert(self_dentry.path(), self_dentry.clone());

    // touch /proc/self/exe
    let exe_dentry = ExeDentry::new(Some(root_dentry.clone()));
    let exe_inode = ExeInode::new(sb.clone().unwrap());
    exe_dentry.set_inode(exe_inode);
    self_dentry.add_child(exe_dentry.clone());
    DCACHE.lock().insert(exe_dentry.path(), exe_dentry.clone());

    // touch /proc/meminfo
    let mem_dentry = MemInfoDentry::new("meminfo", Some(root_dentry.clone()));
    let mem_inode = MemInfoInode::new(sb.clone().unwrap());
    mem_dentry.set_inode(mem_inode);
    root_dentry.add_child(mem_dentry.clone());
    DCACHE.lock().insert(mem_dentry.path(), mem_dentry.clone());

    // touch /proc/mounts
    let mounts_dentry = MountsDentry::new("mounts", Some(root_dentry.clone()));
    let mounts_inode = MountsInode::new(sb.clone().unwrap());
    mounts_dentry.set_inode(mounts_inode);
    root_dentry.add_child(mounts_dentry.clone());
    DCACHE.lock().insert(mounts_dentry.path(), mounts_dentry.clone());

}
