//! (FileWrapper + VfsNodeOps) -> OSInodeInner
//! OSInodeInner -> OSInode
extern crate lwext4_rust;
extern crate virtio_drivers;

use lwext4_rust::InodeTypes;

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};


use crate::drivers::block::BLOCK_DEVICE;
use crate::fs::vfs::{Inode, DCACHE};
use crate::fs::FS_MANAGER;

use alloc::vec;
use alloc::{format, vec::Vec};
use alloc::string::String;
use alloc::boxed::Box;

use super::dentry;
use super::inode::Ext4Inode;
use super::disk::Disk;

use crate::fs::vfs::{File, FileInner};
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use bitflags::*;
use lazy_static::*;

use log::*;
use crate::logging;


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
    /// Construct an OS inode from a Inode
    pub fn new(readable: bool, writable: bool, inode: Arc<dyn Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: UPSafeCell::new(FileInner { offset: 0, inode }) ,
        }
    }

    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer).unwrap();
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

impl File for Ext4File {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice).unwrap();
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice).unwrap();
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}

bitflags! {
    ///Open file flags
    pub struct OpenFlags: u32 {
        ///Read only
        const RDONLY = 0;
        ///Write only
        const WRONLY = 1 << 0;
        ///Read & Write
        const RDWR = 1 << 1;
        ///Allow create
        const CREATE = 1 << 9;
        ///Clear file and return an empty one
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

/// helper function: Open file in ext4 fs with flags
/// @path: absolute path
pub fn open_file(path: &str, flags: OpenFlags) -> Option<Arc<Ext4File>> {
    //let root = FS_MANAGER.lock().get("ext4").unwrap().root();
    let (readable, writable) = flags.read_write();

    // get the root dentry and look up for the inode first
    let root_dentry = {
        let dcache = DCACHE.lock();
        Arc::clone(dcache.get("/").unwrap())
    };
    let root_inode = root_dentry.inode().unwrap();
    
    if flags.contains(OpenFlags::CREATE) {
        if let Some(dentry) = root_dentry.find(path) {
            // clear size
            let inode = dentry.inode().unwrap();
            inode.truncate(0).expect("Error when truncating inode");
            Some(Arc::new(Ext4File::new(readable, writable, inode)))
        } else {
            // create file
            root_inode
                .create(path, InodeTypes::EXT4_DE_REG_FILE)
                .map(|inode| Arc::new(Ext4File::new(readable, writable, inode)))
        }
    } else {
        if let Some(dentry) = root_dentry.find(path) {
            // get the dentry and it is valid (see dentry::find)
            let inode = dentry.inode().unwrap();
            if flags.contains(OpenFlags::TRUNC) {
                inode.truncate(0).expect("Error when truncating inode");
            }
            Some(Arc::new(Ext4File::new(readable, writable, inode)))
        } else {
            None
        }
        
    }
}

/// helper function: List all files in the ext4 filesystem
pub fn list_apps() {
    let root = FS_MANAGER.lock().get("ext4").unwrap().root();
    println!("/**** APPS ****");
    for app in root.ls() {
        println!("{}", app);
    }
    println!("**************/");
}