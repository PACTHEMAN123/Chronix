//! (FileWrapper + VfsNodeOps) -> OSInodeInner
//! OSInodeInner -> OSInode

use core::sync::atomic::AtomicUsize;

use async_trait::async_trait;
use hal::println;


use crate::fs::page::page::PAGE_SIZE;
use crate::fs::vfs::dentry::global_find_dentry;
use crate::fs::vfs::file::SeekFrom;
use crate::fs::vfs::inode::InodeMode;
use crate::fs::vfs::{Dentry, DentryState, Inode, DCACHE};
use crate::fs::FS_MANAGER;
use crate::sync::mutex::SpinNoIrqLock;
use crate::utils::{abs_path_to_name, abs_path_to_parent};

use alloc::vec;
use alloc::{format, vec::Vec};
use alloc::string::String;
use alloc::boxed::Box;

use super::{dentry, Ext4Dentry};
use super::inode::Ext4Inode;
use super::disk::Disk;

use crate::fs::{
    vfs::{File, FileInner},
    OpenFlags,
};
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use bitflags::*;
use lazy_static::*;

use log::*;


/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct Ext4File {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<FileInner>,
}

unsafe impl Send for Ext4File {}
unsafe impl Sync for Ext4File {}

impl Ext4File {
    /// Construct an Ext4File from a dentry
    pub fn new(readable: bool, writable: bool, dentry: Arc<dyn Dentry>) -> Self {
        Self {
            readable,
            writable,
            inner: UPSafeCell::new(FileInner { 
                offset: AtomicUsize::new(0), 
                dentry, 
                flags: SpinNoIrqLock::new(OpenFlags::empty()), 
            }),
        }
    }

    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let inode = self.dentry().unwrap().inode().unwrap();
        let mut buffer = [0u8; PAGE_SIZE];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inode.clone().cache_read_at(self.pos(), &mut buffer).unwrap();
            if len == 0 {
                break;
            }
            self.seek(SeekFrom::Current(len as i64)).expect("seek failed");
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

#[async_trait]
impl File for Ext4File {
    fn inner(&self) -> &FileInner {
        self.inner.exclusive_access()
    }
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    async fn read(&self, buf: &mut [u8]) -> usize {
        let inode = self.dentry().unwrap().inode().unwrap();

        let size = inode.read_at(self.pos(), buf).unwrap();
        self.seek(SeekFrom::Current(size as i64)).expect("seek failed");
        size
    }
    async fn write(&self, buf: &[u8]) -> usize {
        let inode = self.dentry().unwrap().inode().unwrap();
        
        let size = inode.write_at(self.pos(), buf).unwrap();
        self.seek(SeekFrom::Current(size as i64)).expect("seek failed");
        size
    }
}


