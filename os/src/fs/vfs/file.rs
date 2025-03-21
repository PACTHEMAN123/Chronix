//! virtual file system file object

use crate::mm::UserBuffer;
use async_trait::async_trait;

use alloc::{
    sync::Arc,
    boxed::Box,
};
use super::{Dentry, Inode};

/// basic File object
pub struct FileInner {
    /// the dentry it points to
    pub dentry: Arc<dyn Dentry>,
    /// the current pos 
    pub offset: usize,
}

#[async_trait]
/// File trait
pub trait File: Send + Sync {
    /// get basic File object
    fn inner(&self) -> &FileInner;
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    async fn read(&self, mut buf: UserBuffer) -> usize;
    /// Write `UserBuffer` to file
    async fn write(&self, buf: UserBuffer) -> usize;
    /// get the dentry it points to
    fn dentry(&self) -> Option<Arc<dyn Dentry>> {
        Some(self.inner().dentry.clone())
    }
    /// quicker way to get the inode it points to
    /// notice that maybe unsafe!
    fn inode(&self) -> Option<Arc<dyn Inode>> {
        self.dentry().unwrap().inode().clone()
    }
}