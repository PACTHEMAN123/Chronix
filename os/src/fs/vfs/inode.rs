//! VFS Inode

use core::{ops::Range, sync::atomic::{AtomicI32, AtomicU32, AtomicUsize, Ordering}};

use alloc::{string::String, sync::{Arc, Weak}, vec::Vec};

use super::SuperBlock;
use crate::{fs::{page::{cache::PageCache, page::Page}, Xstat, XstatMask}, generate_atomic_accessors, generate_lock_accessors, generate_with_methods, sync::mutex::SpinNoIrqLock, syscall::SysError, timer::ffi::TimeSpec};
use crate::fs::Kstat;

/// the base Inode of all file system
pub struct InodeInner {
    /// inode number
    pub ino: usize,
    /// super block that owned it
    pub super_block: Option<Weak<dyn SuperBlock>>,
    /// size of the file in bytes
    pub size: AtomicUsize,
    /// link count
    pub nlink: AtomicUsize,
    /// owner
    pub uid: AtomicU32,
    /// group
    pub gid: AtomicU32,
    /// mode of inode
    pub mode: SpinNoIrqLock<InodeMode>,
    /// last access time
    pub atime: SpinNoIrqLock<TimeSpec>,
    /// last modification time
    pub mtime: SpinNoIrqLock<TimeSpec>,
    #[allow(unused)]
    /// last state change time(todo: support state change)
    pub ctime: SpinNoIrqLock<TimeSpec>,
}

impl InodeInner {
    /// create a inner using super block
    pub fn new(super_block: Option<Weak<dyn SuperBlock>>, mode: InodeMode, size: usize) -> Self {
        Self {
            ino: inode_alloc(),
            super_block: super_block,
            size: AtomicUsize::new(size),
            nlink: AtomicUsize::new(1),
            uid: AtomicU32::new(0),
            gid: AtomicU32::new(0),
            mode: SpinNoIrqLock::new(mode),
            atime: SpinNoIrqLock::new(TimeSpec::default()),
            mtime: SpinNoIrqLock::new(TimeSpec::default()),
            ctime: SpinNoIrqLock::new(TimeSpec::default()),
        }
    }
    generate_atomic_accessors!(
        uid: u32,
        gid: u32,
        size: usize,
        nlink: usize
    );
    generate_lock_accessors!(
        mode: InodeMode,
        atime: TimeSpec,
        mtime: TimeSpec,
        ctime: TimeSpec
    );
}

/// Inode trait for all file system to implement
pub trait Inode {
    /// return inner
    fn inode_inner(&self) -> &InodeInner {
        todo!()
    }
    /// return Inode type
    fn inode_type(&self) -> InodeMode {
        self.inode_inner().mode().get_type()
    }
    /// use name to lookup under the current inode
    fn lookup(&self, _name: &str) -> Option<Arc<dyn Inode>> {
        todo!()
    }
    /// list all files/dir/symlink under current inode
    fn ls(&self) -> Vec<String> {
        todo!()
    }
    /// read at given offset in direct IO
    /// the Inode should make sure stop reading when at EOF itself
    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> Result<usize, i32> {
        Ok(0)
    }
    /// write at given offset in direct IO
    /// the Inode should make sure stop writing when at EOF itself
    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize, i32> {
        Ok(0)
    }
    /// get the page cache it owned
    fn cache(&self) -> Arc<PageCache> {
        todo!()
    }
    /// get a page at given offset
    /// if the page already in cache, just return the cache
    /// if the page is not in cache, need to load the page into cache
    /// if the offset is out of bound, return None 
    fn read_page_at(self: Arc<Self>, _offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
    /// read at given offset, allowing page caching
    fn cache_read_at(self: Arc<Self>, _offset: usize, _buf: &mut [u8]) -> Result<usize, i32> {
        todo!()
    }
    /// write at given offset, allowing page caching
    fn cache_write_at(self: Arc<Self>, _offset: usize, _buf: &[u8]) -> Result<usize, i32> {
        todo!()
    }
    /// create inode under current inode
    fn create(&self, _name: &str, _mode: InodeMode) -> Option<Arc<dyn Inode>> {
        todo!()
    }
    /// resize the current inode
    fn truncate(&self, _size: usize) -> Result<usize, SysError> {
        todo!()
    }
    /// get attributes of a file
    fn getattr(&self) -> Kstat {
        todo!()
    }
    /// get extra attributes of a file
    fn getxattr(&self, _mask: XstatMask) -> Xstat {
        todo!()
    }
    /// create a symlink of this inode and return the symlink inode
    /// create a inode in link path [link_path]--->[target_path]
    fn symlink(&self, _target_path: &str, _link_path: &str) -> Result<Arc<dyn Inode>, SysError> {
        todo!()
    }
    /// create a hard link using this inode path and the target path
    fn link(&self, _target: &str) -> Result<usize, SysError> {
        todo!()
    }
    /// read out the path from the symlink
    fn readlink(&self) -> Result<String, SysError> {
        todo!()
    }
    /// called by the unlink system call
    fn unlink(&self) -> Result<usize, i32> {
        todo!()
    }
    /// remove inode current inode
    fn remove(&self, _name: &str, _mode: InodeMode) -> Result<usize, i32> {
        todo!()
    }
    /// rename inode from current path to dst path
    /// return the new inode
    fn rename(&self, _target: &str, _new_inode: Option<Arc<dyn Inode>>) -> Result<(), SysError> {
        Err(SysError::EINVAL)
    }
    /// set all cached pages clean when unlink
    fn clean_cached(&self) {
        // do nothing
    }
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
