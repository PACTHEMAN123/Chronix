//! virtual file system file object

use crate::mm::UserBuffer;

use alloc::sync::Arc;
use super::Inode;

/// basic File object
pub struct FileInner {
    /// the inode it points to
    pub inode: Arc<dyn Inode>,
    /// the current pos 
    pub offset: usize,
}

/// File trait
pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> usize;
}