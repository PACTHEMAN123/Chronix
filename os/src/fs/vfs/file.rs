//! virtual file system file object

use core::any::Any;

use crate::{fs::{page::page::PAGE_SIZE, vfs::{dentry::global_find_dentry, inode::InodeMode, DentryState}, OpenFlags}, mm::UserBuffer, syscall::{SysError, SysResult}, utils::{abs_path_to_name, abs_path_to_parent}};
use async_trait::async_trait;

use alloc::{
    boxed::Box, sync::Arc, vec::Vec
};
use downcast_rs::{impl_downcast, Downcast, DowncastSync};
use log::info;
use hal::println;
use super::{Dentry, Inode, DCACHE};

/// basic File object
pub struct FileInner {
    /// the dentry it points to
    pub dentry: Arc<dyn Dentry>,
    /// the current pos 
    pub offset: usize,
}

#[async_trait]
/// File trait
pub trait File: Send + Sync + DowncastSync {
    /// get basic File object
    fn inner(&self) -> &FileInner;
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    async fn read(&self, buf: &mut [u8]) -> usize;
    /// Write `UserBuffer` to file
    async fn write(&self, buf: &[u8]) -> usize;
    /// get the dentry it points to
    fn dentry(&self) -> Option<Arc<dyn Dentry>> {
        Some(self.inner().dentry.clone())
    }
    /// quicker way to get the inode it points to
    /// notice that maybe unsafe!
    fn inode(&self) -> Option<Arc<dyn Inode>> {
        self.dentry().unwrap().inode().clone()
    }
    /// call by ioctl syscall
    fn ioctl(&self, _cmd: usize, _arg: usize) -> SysResult {
        Err(SysError::ENOTTY)
    }
}

impl dyn File {
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut offset = 0usize;
        let inode = self.dentry().unwrap().inode().unwrap();
        let mut buffer = [0u8; PAGE_SIZE];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inode.clone().read_at(offset, &mut buffer).unwrap();
            if len == 0 {
                break;
            }
            offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        //info!("read total size: {}", v.len());
        v
    }
}

/// helper function: Open file in disk fs with flags
/// notice that ext4 file is a abstract
/// it can be reg_file, dir or anything...
/// @path: absolute path
pub fn open_file(path: &str, flags: OpenFlags) -> Option<Arc<dyn File>> {
    //info!("try to open file: {}", path);
    // get the root dentry and look up for the inode first
    let root_dentry = {
        let dcache = DCACHE.lock();
        Arc::clone(dcache.get("/").unwrap())
    };
    
    if flags.contains(OpenFlags::CREATE) {
        if let Some(dentry) = root_dentry.find(path) {
            // clear size
            let inode = dentry.inode().unwrap();
            inode.truncate(0).expect("Error when truncating inode");
            dentry.open(flags)
        } else {
            // create file (todo: now only support root create)
            let name = abs_path_to_name(&path).unwrap();
            let parent_path = abs_path_to_parent(&path).unwrap();
            let parent_dentry = global_find_dentry(&parent_path);
            assert!(parent_dentry.state() == DentryState::USED);
            let inode = parent_dentry.inode().unwrap().create(&name, InodeMode::FILE).unwrap();
            let dentry = parent_dentry.new(&name, parent_dentry.superblock(), Some(parent_dentry.clone()));
            dentry.set_state(DentryState::USED);
            dentry.set_inode(inode);
            dentry.open(flags)
        }
    } else {
        if let Some(dentry) = root_dentry.find(path) {
            // get the dentry and it is valid (see dentry::find)
            let inode = dentry.inode().unwrap();
            if flags.contains(OpenFlags::TRUNC) {
                inode.truncate(0).expect("Error when truncating inode");
            }
            dentry.open(flags)
        } else {
            None
        }
        
    }
}

impl_downcast!(sync File);

/// helper function: List all files in the ext4 filesystem
pub fn list_apps() {
    let root_dentry = {
        let dcache = DCACHE.lock();
        Arc::clone(dcache.get("/").unwrap())
    };
    let root_inode = root_dentry.inode().unwrap();
    println!("/**** APPS ****");
    for app in root_inode.ls() {
        println!("{}", app);
    }
    println!("**************/");
}