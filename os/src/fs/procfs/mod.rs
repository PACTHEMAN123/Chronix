//! proc file system

use alloc::sync::{Arc, Weak};
use meminfo::MemInfoInode;
use mounts::MountsInode;
use self_::ExeInode;

use crate::fs::{procfs::{meminfo::MEM_INFO, mounts::list_mounts}, tmpfs::{dentry::TmpDentry, inode::TmpInode}, vfs::{inode::InodeMode, Inode}, SuperBlock};

use super::vfs::{Dentry, DCACHE};

pub mod fstype;
pub mod superblock;
pub mod self_;
pub mod mounts;
pub mod meminfo;

/// init the whole /proc
pub fn init_procfs(root_dentry: Arc<dyn Dentry>) {
    let sb = root_dentry.inode().unwrap().inode_inner().super_block.clone();

    // mkdir /proc/self
    let self_dentry = create_sys_dir("self", sb.clone().unwrap(), root_dentry.clone());

    // touch /proc/self/exe
    let exe_dentry = TmpDentry::new("exe", Some(root_dentry.clone()));
    let exe_inode = ExeInode::new(sb.clone().unwrap());
    exe_dentry.set_inode(exe_inode);
    self_dentry.add_child(exe_dentry.clone());
    DCACHE.lock().insert(exe_dentry.path(), exe_dentry.clone());

    // touch /proc/meminfo
    create_sys_file(&MEM_INFO.lock().serialize(), "meminfo", sb.clone().unwrap(), root_dentry.clone());
    // touch /proc/mounts
    create_sys_file(&list_mounts(),"mounts", sb.clone().unwrap(), root_dentry.clone());
    // touch /proc/sys/kernel/pid_max
    let sys_dentry = create_sys_dir("sys", sb.clone().unwrap(), root_dentry.clone());
    let kernel_dentry = create_sys_dir("kernel", sb.clone().unwrap(), sys_dentry);
    create_sys_file("4194304", "pid_max", sb.clone().unwrap(), kernel_dentry);
}

/// helper method to generate written dir
pub fn create_sys_dir(name: &str, sb: Weak<dyn SuperBlock>, parent: Arc<dyn Dentry>) -> Arc<dyn Dentry> {
    let dentry = TmpDentry::new(name, Some(parent.clone()));
    let inode = TmpInode::new(sb.clone(), InodeMode::DIR);
    dentry.set_inode(inode);
    parent.add_child(dentry.clone());
    dentry
}

/// helper method to generate written file
pub fn create_sys_file(contents: &str, name: &str, sb: Weak<dyn SuperBlock>, parent: Arc<dyn Dentry>) -> Arc<dyn Dentry> {
    let contents = contents.as_bytes();
    let dentry = TmpDentry::new(name, Some(parent.clone()));
    let inode = TmpInode::new(sb.clone(), InodeMode::FILE);
    let _ = inode.clone().cache_write_at(0, contents);
    dentry.set_inode(inode);
    parent.add_child(dentry.clone());
    dentry
}