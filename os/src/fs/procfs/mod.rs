//! proc file system

use alloc::sync::{Arc, Weak};
use self_::ExeInode;

use crate::fs::{fs::CNXFS, procfs::{meminfo::{MemInfo, MEM_INFO}, mounts::{list_mounts, MountInfo}, sys::kernel::PidMax}, tmpfs::{dentry::TmpDentry, inode::{InodeContent, TmpInode, TmpSysInode}}, vfs::{inode::InodeMode, Inode}, SuperBlock};

use super::vfs::{Dentry, DCACHE};

pub mod fstype;
pub mod superblock;
pub mod self_;
pub mod mounts;
pub mod meminfo;
pub mod sys;

/// init the whole /proc
pub fn init_procfs(root_dentry: Arc<dyn Dentry>) {
    let sb = root_dentry.inode().unwrap().inode_inner().super_block.clone();

    // mkdir /proc/self
    let self_dentry = CNXFS::create_sys_dir("self", sb.clone().unwrap(), root_dentry.clone());

    // touch /proc/self/exe
    let exe_dentry = TmpDentry::new("exe", Some(root_dentry.clone()));
    let exe_inode = ExeInode::new(sb.clone().unwrap());
    exe_dentry.set_inode(exe_inode);
    self_dentry.add_child(exe_dentry.clone());
    DCACHE.lock().insert(exe_dentry.path(), exe_dentry.clone());

    // touch /proc/meminfo
    CNXFS::create_sys_file(Arc::new(MemInfo::new()), "meminfo", root_dentry.clone());
    // touch /proc/mounts
    CNXFS::create_sys_file(Arc::new(MountInfo::new()),"mounts", root_dentry.clone());
    // touch /proc/sys/kernel/pid_max
    let sys_dentry = CNXFS::create_sys_dir("sys", sb.clone().unwrap(), root_dentry.clone());
    let kernel_dentry = CNXFS::create_sys_dir("kernel", sb.clone().unwrap(), sys_dentry);
    CNXFS::create_sys_file(Arc::new(PidMax::new()), "pid_max", kernel_dentry);
}