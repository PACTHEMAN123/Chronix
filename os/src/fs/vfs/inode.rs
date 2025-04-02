//! VFS Inode

use core::{ops::Range, sync::atomic::{AtomicUsize, Ordering}};

use alloc::{collections::btree_map::BTreeMap, string::String, sync::{Arc, Weak}, vec::Vec};
use hal::{addr::{PhysAddrHal, PhysPageNumHal, VirtPageNum, VirtPageNumHal}, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::MapPerm, println};
use range_map::RangeMap;
use xmas_elf::reader::Reader;

use super::SuperBlock;
use crate::{fs::{page::{cache::PageCache, page::Page}, Xstat, XstatMask}, mm::{vm::{KernVmArea, KernVmSpaceHal}, INIT_VMSPACE}, sync::{mutex::{spin_mutex::SpinMutex, Spin}, UPSafeCell}, timer::ffi::TimeSpec};
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
    /// get the page cache it owned
    fn cache(&self) -> Arc<PageCache>;
    /// get a page at given offset
    /// if the page already in cache, just return the cache
    /// if the page is not in cache, need to load the page into cache
    /// if the offset is out of bound, return None 
    fn read_page_at(self: Arc<Self>, offset: usize) -> Option<Arc<Page>>;
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
    /// get extra attributes of a file
    fn getxattr(&self, mask: XstatMask) -> Xstat;
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

pub struct FileReader<T: Inode + ?Sized> {
    inode: Arc<T>,
    mapped: UPSafeCell<RangeMap<usize, Range<VirtPageNum>>>,
}

impl<T: Inode + ?Sized> FileReader<T> {
    pub fn new(inode: Arc<T>) -> Self {
        Self { 
            inode,
            mapped: UPSafeCell::new(RangeMap::new())
        }
    }
}

impl<T: Inode + ?Sized> Reader for FileReader<T> {
    fn len(&self) -> usize {
        self.inode.getattr().st_size as usize
    }

    fn read(&self, offset: usize, len: usize) -> &[u8] {
        const MASK: usize = (1 << Constant::PAGE_SIZE_BITS) - 1;

        let mut start = offset & !MASK;
        let mut end = (offset + len - 1 + Constant::PAGE_SIZE) & !MASK;

        loop {
            
            if let Some((range, range_vpn)) = self.mapped
                .exclusive_access()
                .range_contain_key_value(start..end) 
            {
                let area_offset = offset - range.start;
                return unsafe { 
                    core::slice::from_raw_parts(
                        (range_vpn.start.start_addr().0 + area_offset) as *const u8,
                        len
                    )
                };
            }

            loop {
                match self.mapped.exclusive_access().try_insert(start..end, 0.into()..0.into()) {
                    Ok(range_vpn_ref) => {
                        let mut frames = Vec::new();
                        for offset in (start..end).step_by(Constant::PAGE_SIZE) {
                            let page = self.inode.clone().read_page_at(offset).unwrap();
                            frames.push(page.frame().clone());
                        }
                        
                        let range_vpn = INIT_VMSPACE.lock().map_vm_area(frames, MapPerm::R).unwrap();
                        for vpn in range_vpn.clone() {
                            unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                        }

                        *range_vpn_ref = range_vpn;
                        break;
                    },
                    Err(_) => {
                        while let Some((range, range_vpn)) = self.mapped
                            .exclusive_access()
                            .range_contain_key_value(start..end) 
                        {
                            start = core::cmp::min(start, range.start);
                            end = core::cmp::max(end, range.end);
                            INIT_VMSPACE.lock().unmap_vm_area(range_vpn.clone());
                            self.mapped.exclusive_access().force_remove_one(range);
                        }
                    },
                }
            }
        }
    }
}

impl<T: Inode + ?Sized> Drop for FileReader<T> {
    fn drop(&mut self) {
        for (_, range_vpn) in self.mapped.exclusive_access().iter() {
            INIT_VMSPACE.lock().unmap_vm_area(range_vpn.clone());
        }
    }
}
