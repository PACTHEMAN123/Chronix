//! virtual file system file object

use crate::mm::UserBuffer;
use async_trait::async_trait;

use alloc::{
    sync::Arc,
    boxed::Box,
};
use super::Inode;

/// basic File object
pub struct FileInner {
    /// the inode it points to
    pub inode: Arc<dyn Inode>,
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
    /// get the inode it points to
    fn inode(&self) -> Option<Arc<dyn Inode>> {
        Some(self.inner().inode.clone())
    }
}