//! /proc/mounts file

use core::cmp;

use alloc::{string::{String, ToString}, sync::{Arc, Weak}};
use async_trait::async_trait;
use alloc::boxed::Box;

use crate::{config::BLOCK_SIZE, fs::{tmpfs::inode::InodeContent, vfs::{inode::InodeMode, Dentry, DentryInner, File, FileInner, Inode, InodeInner}, Kstat, OpenFlags, StatxTimestamp, SuperBlock, Xstat, XstatMask, FS_MANAGER}, sync::mutex::SpinNoIrqLock, syscall::SysError};

pub struct MountInfo;

impl MountInfo {
    pub fn new() -> Self {
        Self {}
    }
}

impl InodeContent for MountInfo {
    fn serialize(&self) -> String {
        let mut res = "".to_string();
        let fs_manager = FS_MANAGER.lock();
        for (_, fs) in fs_manager.iter() {
            let sbs = fs.inner().supers.lock();
            for (mount_path, _) in sbs.iter() {
                // device name: (todo)
                res += "device";
                res += " ";
                // mount point
                res += mount_path;
                res += " ";
                // fs type name
                res += fs.name();
                res += " ";
                // fs stat flags (todo)
                res += "rw,nosuid,nodev,noexec,relatime";
                res += " ";
                
                res += "0 0\n";
            }
        }
        res
    }
}

pub fn list_mounts() -> String {
    let mut res = "".to_string();
    let fs_manager = FS_MANAGER.lock();
    for (_, fs) in fs_manager.iter() {
        let sbs = fs.inner().supers.lock();
        for (_mount_path, _) in sbs.iter() {
            // device name: (todo)
            res += "device";
            res += " ";
            // mount point
            res += "/fake";
            res += " ";
            // fs type name
            res += fs.name();
            res += " ";
            // fs stat flags (todo)
            res += "rw,nosuid,nodev,noexec,relatime";
            res += " ";
            
            res += "0 0\n";
        }
    }
    res
}