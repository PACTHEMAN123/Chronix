//! file system module: offer the file system interface
//! define the file trait
//! impl File for OSInode in `inode.rs`
//! impl Stdin and Stdout in `stdio.rs`
#![allow(missing_docs)]
pub mod stdio;
pub mod ext4;
pub mod vfs;

use log::*;
use crate::logging;
pub use stdio::{Stdin, Stdout};

use alloc::{collections::btree_map::BTreeMap, string::{String, ToString}, sync::Arc};

use crate::{drivers::BLOCK_DEVICE, sync::mutex::{SpinNoIrq, SpinNoIrqLock}};
pub use ext4::Ext4SuperBlock;
pub use vfs::{SuperBlock, SuperBlockInner};

/// file system manager
/// hold the lifetime of all file system
/// maintain the mapping
pub static FS_MANAGER: SpinNoIrqLock<BTreeMap<String, Arc<dyn SuperBlock>>> =
    SpinNoIrqLock::new(BTreeMap::new());

/// the default filesystem on disk
pub const DISK_FS_NAME: &str = "ext4";

/// init the file system
pub fn init() {
    // create the ext4 file system using the block device
    let ext4_superblock = Ext4SuperBlock::new(
        SuperBlockInner::new(Some(BLOCK_DEVICE.clone())));
    FS_MANAGER.lock().insert(DISK_FS_NAME.to_string(), ext4_superblock);
    info!("ext4 finish init");
}

/// AT_FDCWD: a special value
pub const AT_FDCWD: isize = -100;

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
