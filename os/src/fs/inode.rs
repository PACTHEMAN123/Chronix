//! (FileWrapper + VfsNodeOps) -> OSInodeInner
//! OSInodeInner -> OSInode
extern crate lwext4_rust;
extern crate virtio_drivers;

use lwext4_rust::InodeTypes;

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};


use crate::drivers::block::BLOCK_DEVICE;

use alloc::vec;
use alloc::{format, vec::Vec};
use alloc::boxed::Box;

use super::ext4fs::{Ext4FileSystem, Inode};
use super::disk::Disk;

use super::File;
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use bitflags::*;
use lazy_static::*;

use log::*;
use crate::logging;

/// The OS inode inner in 'UPSafeCell'
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}

impl OSInode {
    /// Construct an OS inode from a Inode
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }

    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let inner = unsafe {
            self.inner.exclusive_access()
        };
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

lazy_static! {
    /// ext4 file system
    pub static ref EXT4_FS: Arc<Ext4FileSystem> = Arc::new(Ext4FileSystem::new(Disk::new(BLOCK_DEVICE.clone())));

    /// root inode
    pub static ref ROOT_INODE: Arc<Inode> = EXT4_FS.root_dir();
}



impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let inner = unsafe{self.inner.exclusive_access()};
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
        let inner = unsafe{self.inner.exclusive_access()};
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

///Open file with flags
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.lookup(name) {
            // clear size
            inode.truncate(0).expect("Error when truncating inode");
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            ROOT_INODE
                .create(name, InodeTypes::EXT4_DE_REG_FILE)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        ROOT_INODE.lookup(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.truncate(0).expect("Error when truncating inode");
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}