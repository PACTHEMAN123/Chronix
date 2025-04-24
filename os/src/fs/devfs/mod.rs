//! device file system
//! the inode implement maybe different
//! since we have different kinds of devices
//! the dentry (can be seen as dir) and dir inode will be same

use alloc::{collections::btree_map::BTreeMap, string::String, sync::Arc};
use fatfs::info;
use null::{NullDentry, NullInode};
use rtc::{RtcDentry, RtcInode};
use tty::{TtyDentry, TtyFile, TtyInode, TTY};
use urandom::{UrandomDentry, UrandomInode};
use zero::{ZeroDentry, ZeroInode};

use crate::sync::mutex::SpinNoIrqLock;

use super::{vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, Inode, InodeInner, DCACHE}, OpenFlags, SuperBlock};

pub mod tty;
pub mod null;
pub mod superblock;
pub mod fstype;
pub mod rtc;
pub mod urandom;
pub mod zero;

/// init the whole /dev
pub fn init_devfs(root_dentry: Arc<dyn Dentry>) {
    let sb = root_dentry.superblock();

    // add /dev/tty
    let tty_dentry = TtyDentry::new("tty", sb.clone(), Some(root_dentry.clone()));
    let tty_inode = TtyInode::new(sb.clone());
    tty_dentry.set_inode(tty_inode);
    root_dentry.add_child(tty_dentry.clone());
    log::debug!("dcache insert: {}", tty_dentry.path());
    DCACHE.lock().insert(tty_dentry.path(), tty_dentry.clone());
    let tty_file = TtyFile::new(tty_dentry);
    TTY.call_once(|| tty_file);

    // add /dev/null
    let null_dentry = NullDentry::new("null", sb.clone(), Some(root_dentry.clone()));
    let null_inode = NullInode::new(sb.clone());
    null_dentry.set_inode(null_inode);
    root_dentry.add_child(null_dentry.clone());
    log::debug!("dcache insert: {}", null_dentry.path());
    DCACHE.lock().insert(null_dentry.path(), null_dentry.clone());

    // add /dev/rtc
    let rtc_dentry = RtcDentry::new("rtc", sb.clone(), Some(root_dentry.clone()));
    let rtc_inode = RtcInode::new(sb.clone());
    rtc_dentry.set_inode(rtc_inode);
    root_dentry.add_child(rtc_dentry.clone());
    log::debug!("dcache insert: {}", rtc_dentry.path());
    DCACHE.lock().insert(rtc_dentry.path(), rtc_dentry.clone());

    // add /dev/urandom
    let urandom_dentry = UrandomDentry::new("urandom", sb.clone(), Some(root_dentry.clone()));
    let urandom_inode = UrandomInode::new(sb.clone());
    urandom_dentry.set_inode(urandom_inode);
    root_dentry.add_child(urandom_dentry.clone());
    log::debug!("dcache insert: {}", urandom_dentry.path());
    DCACHE.lock().insert(urandom_dentry.path(), urandom_dentry.clone());

    // add /dev/zero
    let zero_dentry = ZeroDentry::new("zero", sb.clone(), Some(root_dentry.clone()));
    let zero_inode = ZeroInode::new(sb.clone());
    zero_dentry.set_inode(zero_inode);
    root_dentry.add_child(zero_dentry.clone());
    log::debug!("dcache insert: {}", zero_dentry.path());
    DCACHE.lock().insert(zero_dentry.path(), zero_dentry.clone());
    
}





