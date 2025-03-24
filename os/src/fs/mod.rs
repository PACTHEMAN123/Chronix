//! file system module: offer the file system interface
//! define the file trait
//! impl File for OSInode in `inode.rs`
//! impl Stdin and Stdout in `stdio.rs`
#![allow(missing_docs)]
pub mod stdio;
pub mod fat32;
pub mod ext4;
pub mod vfs;
pub mod pipe;
pub mod page;

use ext4::Ext4FSType;
use fatfs::FatType;
use log::*;
pub use stdio::{Stdin, Stdout};

use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::{String, ToString}, sync::Arc};
use vfs::fstype::{FSType, MountFlags};

use crate::{drivers::BLOCK_DEVICE, sync::mutex::{SpinNoIrq, SpinNoIrqLock}};
pub use ext4::Ext4SuperBlock;
pub use vfs::{SuperBlock, SuperBlockInner};

/// file system manager
/// hold the lifetime of all file system
/// maintain the mapping
pub static FS_MANAGER: SpinNoIrqLock<BTreeMap<String, Arc<dyn FSType>>> =
    SpinNoIrqLock::new(BTreeMap::new());

/// the default filesystem on disk
#[cfg(not(feature = "fat32"))]
pub const DISK_FS_NAME: &str = "ext4";

#[cfg(not(feature = "fat32"))]
type DiskFSType = Ext4FSType;

#[cfg(feature = "fat32")]
pub const DISK_FS_NAME: &str = "fat32";

use crate::fs::fat32::fstype::Fat32FSType;
#[cfg(feature = "fat32")]
type DiskFSType = Fat32FSType;


/// register all filesystem
/// we need this to borrow static reference to mount the fs
fn register_all_fs() {
    let diskfs = DiskFSType::new();
    FS_MANAGER.lock().insert(diskfs.name().to_string(), diskfs);
}

/// get the file system by name
pub fn get_filesystem(name: &str) -> &'static Arc<dyn FSType> {
    let arc = FS_MANAGER.lock().get(name).unwrap().clone();
    Box::leak(Box::new(arc))
}


/// init the file system
pub fn init() {
    register_all_fs();
    // create the ext4 file system using the block device
    let diskfs = get_filesystem(DISK_FS_NAME);
    diskfs.mount("/", None, MountFlags::empty(), Some(BLOCK_DEVICE.clone()));
    info!("fs finish init");

}

/// AT_FDCWD: a special value
pub const AT_FDCWD: isize = -100;
/// Remove directory instead of unlinking file.
pub const AT_REMOVEDIR: i32 = 0x200;

bitflags! {
    ///Open file flags
    pub struct OpenFlags: u32 {
        const APPEND = 1 << 10;
        const ASYNC = 1 << 13;
        const DIRECT = 1 << 14;
        const DSYNC = 1 << 12;
        const EXCL = 1 << 7;
        const NOATIME = 1 << 18;
        const NOCTTY = 1 << 8;
        const NOFOLLOW = 1 << 17;
        const PATH = 1 << 21;
        /// TODO: need to find 1 << 15
        const TEMP = 1 << 15;
        /// Read only
        const RDONLY = 0;
        /// Write only
        const WRONLY = 1 << 0;
        /// Read & Write
        const RDWR = 1 << 1;
        /// Allow create
        const CREATE = 1 << 6;
        /// Clear file and return an empty one
        const TRUNC = 1 << 9;
        /// Directory
        const DIRECTORY = 1 << 16;
        /// Enable the close-on-exec flag for the new file descriptor
        const CLOEXEC = 1 << 19;
        /// When possible, the file is opened in nonblocking mode
        const NONBLOCK = 1 << 11;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

// Defined in <bits/struct_stat.h>
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Kstat {
    /// device
    pub st_dev: u64,
    /// inode number
    pub st_ino: u64,
    /// file type
    pub st_mode: u32,
    /// number of hard links
    pub st_nlink: u32,
    /// user id
    pub st_uid: u32,
    /// user group id
    pub st_gid: u32,
    /// device no
    pub st_rdev: u64,
    _pad0: u64,
    /// file size
    pub st_size: i64,
    /// block size
    pub st_blksize: i32,
    _pad1: i32,
    /// number of blocks
    pub st_blocks: i64,
    /// last access time (s)
    pub st_atime_sec: isize,
    /// last access time (ns)
    pub st_atime_nsec: isize,
    /// last modify time (s)
    pub st_mtime_sec: isize,
    /// last modify time (ns)
    pub st_mtime_nsec: isize,
    /// last change time (s)
    pub st_ctime_sec: isize,
    /// last change time (ns)
    pub st_ctime_nsec: isize,
}

// Defined in <sys/utsname.h>.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UtsName {
    /// Name of the implementation of the operating system.
    pub sysname: [u8; 65],
    /// Name of this node on the network.
    pub nodename: [u8; 65],
    /// Current release level of this implementation.
    pub release: [u8; 65],
    /// Current version level of this release.
    pub version: [u8; 65],
    /// Name of the hardware type the system is running on.
    pub machine: [u8; 65],
    /// Name of the domain of this node on the network.
    pub domainname: [u8; 65],
}

impl UtsName {
    pub fn default() -> Self {
        Self {
            sysname: Self::from_str("Linux"),
            nodename: Self::from_str("Linux"),
            release: Self::from_str("5.19.0-42-generic"),
            version: Self::from_str(
                "#43~22.04.1-Ubuntu SMP PREEMPT_DYNAMIC Fri Apr 21 16:51:08 UTC 2",
            ),
            machine: Self::from_str("RISC-V SiFive Freedom U740 SoC"),
            domainname: Self::from_str("localhost"),
        }
    }

    fn from_str(info: &str) -> [u8; 65] {
        let mut data: [u8; 65] = [0; 65];
        data[..info.len()].copy_from_slice(info.as_bytes());
        data
    }
}
