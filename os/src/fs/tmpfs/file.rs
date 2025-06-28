use core::sync::atomic::AtomicUsize;

use alloc::sync::Arc;
use async_trait::async_trait;
use alloc::boxed::Box;

use crate::{fs::{vfs::{file::SeekFrom, Dentry, File, FileInner}, OpenFlags}, sync::{mutex::SpinNoIrqLock, UPSafeCell}, syscall::SysError};


pub struct TmpFile {
    inner: UPSafeCell<FileInner>,
}

unsafe impl Send for TmpFile {}
unsafe impl Sync for TmpFile {}

impl TmpFile {
    /// Construct an TmpFile from a dentry
    pub fn new(dentry: Arc<dyn Dentry>) -> Self {
        Self {
            inner: UPSafeCell::new(FileInner { 
                offset: AtomicUsize::new(0), 
                dentry, 
                flags: SpinNoIrqLock::new(OpenFlags::empty()), 
            }),
        }
    }

    pub fn new_arc(dentry: Arc<dyn Dentry>) -> Arc<Self> {
        Arc::new(Self {
            inner: UPSafeCell::new(FileInner { 
                offset: AtomicUsize::new(0), 
                dentry, 
                flags: SpinNoIrqLock::new(OpenFlags::empty()), 
            }),
        })
    }
}

#[async_trait]
impl File for TmpFile {
    fn file_inner(&self) -> &FileInner {
        self.inner.exclusive_access()
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, SysError> {
        let inode = self.dentry().unwrap().inode().unwrap();
        let size = inode.cache_read_at(offset, buf).unwrap();
        Ok(size)
    }
    async fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, SysError> {
        let inode = self.dentry().unwrap().inode().unwrap();
        let size = inode.cache_write_at(offset, buf).unwrap();
        Ok(size)
    }
    async fn read(&self, buf: &mut [u8]) -> Result<usize, SysError> {
        let inode = self.dentry().unwrap().inode().unwrap();
        log::debug!("[Tmp file] read start from pos {}", self.pos());
        let size = inode.cache_read_at(self.pos(), buf).unwrap();
        self.seek(SeekFrom::Current(size as i64)).expect("seek failed");
        Ok(size)
    }
    async fn write(&self, buf: &[u8]) -> Result<usize, SysError> {
        if self.flags().contains(OpenFlags::O_APPEND) {
            self.set_pos(self.size());
        }
        let pos = self.pos();
        log::debug!("[Tmp file] writing {}, state: {:?}", self.dentry().unwrap().path(), self.dentry().unwrap().state());
        let inode = self.dentry().unwrap().inode().unwrap();
        let size = inode.cache_write_at(pos, buf).unwrap();
        log::debug!("[Tmp file] set pos at {}", pos + size);
        self.set_pos(pos + size);
        Ok(size)
    }
}