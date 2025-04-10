use core::sync::atomic::AtomicUsize;

use alloc::sync::Arc;
use async_trait::async_trait;
use alloc::boxed::Box;

use crate::{fs::{vfs::{file::SeekFrom, Dentry, File, FileInner}, OpenFlags}, sync::mutex::SpinNoIrqLock};


/// simple file system file
pub struct SpFile {
    inner: FileInner,
}

unsafe impl Send for SpFile {}
unsafe impl Sync for SpFile {}

impl SpFile {
    pub fn new(dentry: Arc<dyn Dentry>) -> Arc<Self> {
        Arc::new(Self {
            inner: FileInner { 
                dentry: dentry, 
                offset: AtomicUsize::new(0), 
                flags:  SpinNoIrqLock::new(OpenFlags::empty()),
            }
        })
    }
}

#[async_trait]
impl File for SpFile {
    fn inner(&self) -> &FileInner {
        &self.inner
    }
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        false
    }
    async fn read(&self, _buf: &mut [u8]) -> usize {
        panic!("cannot read sp file")
    }
    async fn write(&self, _buf: &[u8]) -> usize {
        panic!("cannot write sp file")
    }
}