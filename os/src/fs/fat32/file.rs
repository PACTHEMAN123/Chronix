use core::sync::atomic::AtomicUsize;

use alloc::{sync::Arc, vec::Vec, boxed::Box};
use async_trait::async_trait;

use crate::{fs::{page::page::PAGE_SIZE, vfs::{file::SeekFrom, Dentry, File, FileInner}, OpenFlags}, mm::UserBuffer, sync::{mutex::SpinNoIrqLock, UPSafeCell}};


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
            inner: UPSafeCell::new(FileInner { 
                offset: AtomicUsize::new(0), 
                dentry,
                flags: SpinNoIrqLock::new(OpenFlags::empty())
            }) ,
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