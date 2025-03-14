//! VFS Inode

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{string::String, sync::{Arc, Weak}, vec::Vec};
use lwext4_rust::InodeTypes;

use super::SuperBlock;
use crate::timer::ffi::TimeSpec;

/// the base Inode of all file system
pub struct InodeInner {
    /// inode number
    pub ino: usize,
    /// super block that owned it
    pub super_block: Weak<dyn SuperBlock>,
    /// last access time
    pub atime: TimeSpec,
    /// last modification time
    pub mtime: TimeSpec,
    #[allow(unused)]
    /// last state change time(todo: support state change)
    pub ctime: TimeSpec,
}

impl InodeInner {
    /// create a inner using super block
    pub fn new(super_block: Arc<dyn SuperBlock>) -> Self {
        Self {
            ino: inode_alloc(),
            super_block: Arc::downgrade(&super_block),
            atime: TimeSpec::default(),
            mtime: TimeSpec::default(),
            ctime: TimeSpec::default(),
        }
    }
}

/// Inode trait for all file system to implement
pub trait Inode {
    /// return inner
    fn inner(&self) -> &InodeInner;
    /// use name to lookup under the current inode
    fn lookup(&self, name: &str) -> Option<Arc<dyn Inode>>;
    /// list all files/dir/symlink under current inode
    fn ls(&self) -> Vec<String>;
    /// read at given offset
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32>;
    /// write at given offset
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, i32>;
    /// create inode
    fn create(&self, path: &str, ty: InodeTypes) -> Option<Arc<dyn Inode>>;
    /// remove inode
    fn remove(&self, path: &str) -> Result<usize, i32>;
    /// get current inode parent
    fn parent(&self) -> Option<Arc<dyn Inode>>;
    /// rename current inode
    fn rename(&self, src_path: &str, dst_path: &str) -> Result<usize, i32>;
    /// resize the current inode
    fn truncate(&self, size: u64) -> Result<usize, i32>;
}

static INODE_NUMBER: AtomicUsize = AtomicUsize::new(0);

fn inode_alloc() -> usize {
    INODE_NUMBER.fetch_add(1, Ordering::Relaxed)
}