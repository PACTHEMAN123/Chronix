//! device file system
//! the inode implement maybe different
//! since we have different kinds of devices
//! the dentry (can be seen as dir) and dir inode will be same

use alloc::{collections::btree_map::BTreeMap, string::String, sync::Arc};
use fatfs::info;
use null::{NullDentry, NullInode};
use tty::{TtyDentry, TtyFile, TtyInode, TTY};

use crate::sync::mutex::SpinNoIrqLock;

use super::{vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, Inode, InodeInner, DCACHE}, OpenFlags, SuperBlock};

pub mod tty;
pub mod null;
pub mod superblock;
pub mod fstype;

/// init the whole /dev
pub fn init_devfs(root_dentry: Arc<dyn Dentry>) {
    let sb = root_dentry.superblock();

    // add /dev/tty
    let tty_dentry = TtyDentry::new("tty", sb.clone(), Some(root_dentry.clone()));
    let tty_inode = TtyInode::new(sb.clone());
    tty_dentry.set_inode(tty_inode);
    root_dentry.add_child(tty_dentry.clone());
    log::info!("dcache insert: {}", tty_dentry.path());
    DCACHE.lock().insert(tty_dentry.path(), tty_dentry.clone());
    let tty_file = TtyFile::new(tty_dentry);
    TTY.call_once(|| tty_file);

    // add /dev/null
    let null_dentry = NullDentry::new("null", sb.clone(), Some(root_dentry.clone()));
    let null_inode = NullInode::new(sb.clone());
    null_dentry.set_inode(null_inode);
    root_dentry.add_child(null_dentry.clone());
    log::info!("dcache insert: {}", null_dentry.path());
    DCACHE.lock().insert(null_dentry.path(), null_dentry.clone());

    
}





