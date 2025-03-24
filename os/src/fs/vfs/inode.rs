//! VFS Inode

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{string::String, sync::{Arc, Weak}, vec::Vec};

use super::SuperBlock;
use crate::timer::ffi::TimeSpec;
use crate::fs::Kstat;

/// the base Inode of all file system
pub struct InodeInner {
    /// inode number
    pub ino: usize,
    /// super block that owned it
    pub super_block: Weak<dyn SuperBlock>,
    /// size of the file in bytes
    pub size: usize,
    /// link count
    pub nlink: usize,
    /// mode of inode
    pub mode: InodeMode,
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
    pub fn new(super_block: Arc<dyn SuperBlock>, mode: InodeMode, size: usize) -> Self {
        Self {
            ino: inode_alloc(),
            super_block: Arc::downgrade(&super_block),
            size: size,
            nlink: 1,
            mode: mode,
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
    /// read at given offset in direct IO
    /// the Inode should make sure stop reading when at EOF itself
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32>;
    /// write at given offset in direct IO
    /// the Inode should make sure stop writing when at EOF itself
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, i32>;
    /// read at given offset, allowing page caching
    fn cache_read_at(self: Arc<Self>, offset: usize, buf: &mut [u8]) -> Result<usize, i32>;
    /// write at given offset, allowing page caching
    fn cache_write_at(self: Arc<Self>, offset: usize, buf: &[u8]) -> Result<usize, i32>;
    /// create inode under current inode
    fn create(&self, name: &str, mode: InodeMode) -> Option<Arc<dyn Inode>>;
    /// resize the current inode
    fn truncate(&self, size: u64) -> Result<usize, i32>;
    /// get attributes of a file
    fn getattr(&self) -> Kstat;
    /// called by the unlink system call
    fn unlink(&self) -> Result<usize, i32>;
    /// remove inode current inode
    fn remove(&self, name: &str, mode: InodeMode) -> Result<usize, i32>;
}

static INODE_NUMBER: AtomicUsize = AtomicUsize::new(0);

fn inode_alloc() -> usize {
    INODE_NUMBER.fetch_add(1, Ordering::Relaxed)
}

bitflags::bitflags! {
    /// Inode mode(use in kstat)
    pub struct InodeMode: u32 {
        /// Type.
        const TYPE_MASK = 0o170000;
        /// FIFO.
        const FIFO  = 0o010000;
        /// Character device.
        const CHAR  = 0o020000;
        /// Directory
        const DIR   = 0o040000;
        /// Block device
        const BLOCK = 0o060000;
        /// Regular file.
        const FILE  = 0o100000;
        /// Symbolic link.
        const LINK  = 0o120000;
        /// Socket
        const SOCKET = 0o140000;

        /// Set-user-ID on execution.
        const SET_UID = 0o4000;
        /// Set-group-ID on execution.
        const SET_GID = 0o2000;
        /// sticky bit
        const STICKY = 0o1000;
        /// Read, write, execute/search by owner.
        const OWNER_MASK = 0o700;
        /// Read permission, owner.
        const OWNER_READ = 0o400;
        /// Write permission, owner.
        const OWNER_WRITE = 0o200;
        /// Execute/search permission, owner.
        const OWNER_EXEC = 0o100;

        /// Read, write, execute/search by group.
        const GROUP_MASK = 0o70;
        /// Read permission, group.
        const GROUP_READ = 0o40;
        /// Write permission, group.
        const GROUP_WRITE = 0o20;
        /// Execute/search permission, group.
        const GROUP_EXEC = 0o10;

        /// Read, write, execute/search by others.
        const OTHER_MASK = 0o7;
        /// Read permission, others.
        const OTHER_READ = 0o4;
        /// Write permission, others.
        const OTHER_WRITE = 0o2;
        /// Execute/search permission, others.
        const OTHER_EXEC = 0o1;
    }
}
