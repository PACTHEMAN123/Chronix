//! virtual file system file object

use core::{any::Any, sync::atomic::{AtomicUsize, Ordering}, task::Poll};


use crate::{fs::{page::page::PAGE_SIZE, vfs::{dentry::global_find_dentry, inode::InodeMode, DentryState}, OpenFlags}, sync::mutex::{spin_mutex::SpinMutex, SpinNoIrqLock}, syscall::{SysError, SysResult}, utils::{abs_path_to_name, abs_path_to_parent}};
use async_trait::async_trait;

use alloc::{
    boxed::Box, sync::Arc, vec::Vec
};
use downcast_rs::{impl_downcast, Downcast, DowncastSync};
use log::info;
use hal::println;
use xmas_elf::reader::Reader;
use super::{Dentry, Inode, DCACHE};

/// basic File object
pub struct FileInner {
    /// the dentry it points to
    pub dentry: Arc<dyn Dentry>,
    /// the current pos 
    pub offset: AtomicUsize,
    /// file flags
    pub flags: SpinNoIrqLock<OpenFlags>,
}

bitflags! {
    // Defined in <bits/poll.h>.
    pub struct PollEvents: i16 {
        // Event types that can be polled for. These bits may be set in `events' to
        // indicate the interesting event types; they will appear in `revents' to
        // indicate the status of the file descriptor.
        /// There is data to read.
        const IN = 0x001;
        /// There is urgent data to read.
        const PRI = 0x002;
        ///  Writing now will not block.
        const OUT = 0x004;

        // Event types always implicitly polled for. These bits need not be set in
        // `events', but they will appear in `revents' to indicate the status of the
        // file descriptor.
        /// Error condition.
        const ERR = 0x008;
        /// Hang up.
        const HUP = 0x010;
        /// Invalid poll request.
        const INVAL = 0x020;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeekFrom {
    /// set the offset to given index
    Start(u64),
    /// set the offset using current file size
    End(i64),
    /// set the offset using current pos
    Current(i64),
}

#[async_trait]
/// File trait
pub trait File: Send + Sync + DowncastSync {
    /// get basic File object
    fn file_inner(&self) -> &FileInner;
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file, will adjust file offset
    async fn read(&self, buf: &mut [u8]) -> Result<usize, SysError>;
    /// Write file, will adjust file offset
    async fn write(&self, buf: &[u8]) -> Result<usize, SysError>;
    /// Read file, file offset will not change
    async fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> Result<usize, SysError> {
        Err(SysError::EINVAL)
    }
    /// Write file, file offset will not change
    async fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize, SysError> {
        Err(SysError::EINVAL)
    }
    /// get the dentry it points to
    fn dentry(&self) -> Option<Arc<dyn Dentry>> {
        Some(self.file_inner().dentry.clone())
    }
    /// quicker way to get the inode it points to
    /// notice that maybe unsafe!
    fn inode(&self) -> Option<Arc<dyn Inode>> {
        self.dentry().unwrap().inode().clone()
    }
    /// call by ioctl syscall
    fn ioctl(&self, _cmd: usize, _arg: usize) -> SysResult {
        Err(SysError::ENOTTY)
    }
    /// base poll 
    async fn base_poll(&self, events: PollEvents) -> PollEvents{
        let mut res = PollEvents::empty();
        if events.contains(PollEvents::IN) {
            res |= PollEvents::IN
        }
        if events.contains(PollEvents::OUT) {
            res |= PollEvents::OUT;
        }
        res
    }
    /// get the file flags
    fn flags(&self) -> OpenFlags {
        self.file_inner().flags.lock().clone()
    }
    /// set the file flags
    fn set_flags(&self, flags: OpenFlags) {
        *self.file_inner().flags.lock() = flags
    }
    /// the file size 
    /// (this method should only be called when inode is a file)
    fn size(&self) -> usize {
        self.inode().unwrap().getattr().st_size as usize
    }
    /// get file current offset
    fn pos(&self) -> usize {
        self.file_inner().offset.load(Ordering::Relaxed)
    }
    /// set file current offset
    fn set_pos(&self, pos: usize) {
        self.file_inner().offset.store(pos, Ordering::Relaxed);
    }
    /// move the file position index (see lseek)
    /// allows the file offset to be set beyond the end of the
    /// file (but this does not change the size of the file).  If data is
    /// later written at this point, subsequent reads of the data in the
    /// gap (a "hole") return null bytes ('\0') until data is actually
    /// written into the gap.
    fn seek(&self, offset: SeekFrom) -> Result<usize, SysError> {
        let mut pos = self.pos();
        match offset {
            SeekFrom::Current(off) => {
                if off < 0 {
                    if pos as i64 - off.abs() < 0 {
                        return Err(SysError::EINVAL)
                    } else {
                        pos -= off.abs() as usize;
                    }
                } else {
                    pos += off as usize;
                }
            }
            SeekFrom::Start(off) => {
                pos = off as usize;
            }
            SeekFrom::End(off) => {
                let size = self.size();
                if off < 0 {
                    pos = size - off.abs() as usize;
                } else {
                    pos = size + off as usize;
                }
            }
        }
        self.set_pos(pos);
        Ok(pos)
    }
}

impl dyn File {
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut offset = 0usize;
        let inode = self.dentry().unwrap().inode().unwrap();
        let mut buffer = [0u8; PAGE_SIZE];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inode.clone().read_at(offset, &mut buffer).unwrap();
            if len == 0 {
                break;
            }
            offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        //info!("read total size: {}", v.len());
        v
    }
    // given the event and track the event async, returns the event if is ready
    pub async fn poll(&self, events: PollEvents) -> PollEvents {
        self.base_poll(events).await
    }
}

/// helper function: Open file in disk fs with flags
/// notice that ext4 file is a abstract
/// it can be reg_file, dir or anything...
/// @path: absolute path
pub fn open_file(path: &str, flags: OpenFlags) -> Option<Arc<dyn File>> {
    //info!("try to open file: {}", path);
    // get the root dentry and look up for the inode first
    let root_dentry = {
        let dcache = DCACHE.lock();
        Arc::clone(dcache.get("/").unwrap())
    };
    
    if flags.contains(OpenFlags::O_CREAT) {
        if let Some(dentry) = root_dentry.find(path).expect("failed") {
            // clear size
            let inode = dentry.inode().unwrap();
            inode.truncate(0).expect("Error when truncating inode");
            dentry.open(flags)
        } else {
            // create file (todo: now only support root create)
            let name = abs_path_to_name(&path).unwrap();
            let parent_path = abs_path_to_parent(&path).unwrap();
            let parent_dentry = global_find_dentry(&parent_path).expect("no parent");
            assert!(parent_dentry.state() == DentryState::USED);
            let inode = parent_dentry.inode().unwrap().create(&name, InodeMode::FILE).unwrap();
            let dentry = parent_dentry.new(&name, Some(parent_dentry.clone()));
            dentry.set_state(DentryState::USED);
            dentry.set_inode(inode);
            dentry.open(flags)
        }
    } else {
        if let Some(dentry) = root_dentry.find(path).expect("failed") {
            // get the dentry and it is valid (see dentry::find)
            let inode = dentry.inode().unwrap();
            if flags.contains(OpenFlags::O_TRUNC) {
                inode.truncate(0).expect("Error when truncating inode");
            }
            dentry.open(flags)
        } else {
            None
        }
        
    }
}

impl_downcast!(sync File);

/// helper function: List all files in the ext4 filesystem
pub fn list_apps() {
    let root_dentry = {
        let dcache = DCACHE.lock();
        Arc::clone(dcache.get("/").unwrap())
    };
    let root_inode = root_dentry.inode().unwrap();
    println!("/**** APPS ****");
    for app in root_inode.ls() {
        println!("{}", app);
    }
    println!("**************/");
}