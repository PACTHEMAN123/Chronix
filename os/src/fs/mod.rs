//! file system module: offer the file system interface
//! define the file trait
//! impl File for OSInode in `inode.rs`
//! impl Stdin and Stdout in `stdio.rs`
pub mod stdio;
pub mod ext4;
pub mod vfs;

use log::*;
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
