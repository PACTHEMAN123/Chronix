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

use crate::{fs::{devfs::cpu_dma_latency::{CpuDmaLatencyDentry, CpuDmaLatencyInode}, tmpfs::{dentry::TmpDentry, inode::TmpInode}}, sync::mutex::SpinNoIrqLock};

use super::{vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, Inode, InodeInner, DCACHE}, OpenFlags, SuperBlock};

pub mod tty;
pub mod null;
pub mod superblock;
pub mod fstype;
pub mod rtc;
pub mod urandom;
pub mod zero;
pub mod cpu_dma_latency;

/// init the whole /dev
pub fn init_devfs(root_dentry: Arc<dyn Dentry>) {
    let sb = root_dentry.inode().unwrap().inode_inner().super_block.clone();

    // add /dev/tty
    let tty_dentry = TtyDentry::new("tty", Some(root_dentry.clone()));
    let tty_inode = TtyInode::new(sb.clone().unwrap());
    tty_dentry.set_inode(tty_inode);
    root_dentry.add_child(tty_dentry.clone());
    log::debug!("dcache insert: {}", tty_dentry.path());
    DCACHE.lock().insert(tty_dentry.path(), tty_dentry.clone());
    let tty_file = TtyFile::new(tty_dentry);
    TTY.call_once(|| tty_file);

    // add /dev/null
    let null_dentry = NullDentry::new("null", Some(root_dentry.clone()));
    let null_inode = NullInode::new(sb.clone().unwrap());
    null_dentry.set_inode(null_inode);
    root_dentry.add_child(null_dentry.clone());
    log::debug!("dcache insert: {}", null_dentry.path());
    DCACHE.lock().insert(null_dentry.path(), null_dentry.clone());

    // add /dev/rtc
    let rtc_dentry = RtcDentry::new("rtc", Some(root_dentry.clone()));
    let rtc_inode = RtcInode::new(sb.clone().unwrap());
    rtc_dentry.set_inode(rtc_inode);
    root_dentry.add_child(rtc_dentry.clone());
    log::debug!("dcache insert: {}", rtc_dentry.path());
    DCACHE.lock().insert(rtc_dentry.path(), rtc_dentry.clone());

    // add /dev/urandom
    let urandom_dentry = UrandomDentry::new("urandom", Some(root_dentry.clone()));
    let urandom_inode = UrandomInode::new(sb.clone().unwrap());
    urandom_dentry.set_inode(urandom_inode);
    root_dentry.add_child(urandom_dentry.clone());
    log::debug!("dcache insert: {}", urandom_dentry.path());
    DCACHE.lock().insert(urandom_dentry.path(), urandom_dentry.clone());

    // add /dev/zero
    let zero_dentry = ZeroDentry::new("zero", Some(root_dentry.clone()));
    let zero_inode = ZeroInode::new(sb.clone().unwrap());
    zero_dentry.set_inode(zero_inode);
    root_dentry.add_child(zero_dentry.clone());
    log::debug!("dcache insert: {}", zero_dentry.path());
    DCACHE.lock().insert(zero_dentry.path(), zero_dentry.clone());
    
    // add /dev/cpu_dma_latency
    let cpu_dma_latency_dentry = CpuDmaLatencyDentry::new("cpu_dma_latency", Some(root_dentry.clone()));
    let cpu_dma_latency_inode = CpuDmaLatencyInode::new(sb.clone().unwrap());
    cpu_dma_latency_dentry.set_inode(cpu_dma_latency_inode);
    root_dentry.add_child(cpu_dma_latency_dentry.clone());
    log::debug!("dcache insert: {}", cpu_dma_latency_dentry.path());
    DCACHE.lock().insert(cpu_dma_latency_dentry.path(), cpu_dma_latency_dentry.clone());

    // add /dev/shm
    // TODO: now only implement by tmp file
    let shm_dentry = TmpDentry::new("shm", Some(root_dentry.clone()));
    let shm_inode = TmpInode::new(sb.clone().unwrap(), InodeMode::DIR);
    shm_dentry.set_inode(shm_inode);
    root_dentry.add_child(shm_dentry.clone());
    log::debug!("dcache insert: {}", shm_dentry.path());
    DCACHE.lock().insert(shm_dentry.path(), shm_dentry.clone());
}





