use alloc::{sync::Arc, vec::Vec, boxed::Box};
use async_trait::async_trait;

use crate::{fs::{page::page::PAGE_SIZE, vfs::{Dentry, File, FileInner}}, mm::UserBuffer, sync::UPSafeCell};


pub struct FatFile {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<FileInner>,
}

unsafe impl Send for FatFile {}
unsafe impl Sync for FatFile {}

impl FatFile {
    /// Construct an Ext4File from a dentry
    pub fn new(readable: bool, writable: bool, dentry: Arc<dyn Dentry>) -> Self {
        Self {
            readable,
            writable,
            inner: UPSafeCell::new(FileInner { offset: 0, dentry }) ,
        }
    }
}

#[async_trait]
impl File for FatFile {
    fn inner(&self) -> &FileInner {
        self.inner.exclusive_access()
    }
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    async fn read(&self, mut buf: UserBuffer) -> usize {
        let inner = self.inner.exclusive_access();
        let inode = self.dentry().unwrap().inode().unwrap();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inode.read_at(inner.offset, *slice).unwrap();
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    async fn write(&self, buf: UserBuffer) -> usize {
        let inner = self.inner.exclusive_access();
        let inode = self.dentry().unwrap().inode().unwrap();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inode.write_at(inner.offset, *slice).unwrap();
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}