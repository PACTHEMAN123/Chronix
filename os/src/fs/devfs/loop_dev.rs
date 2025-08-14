//! /dev/loop device
//! 

use alloc::sync::Arc;
use async_trait::async_trait;
use alloc::boxed::Box;
use strum::FromRepr;

use crate::{config::BLOCK_SIZE, devices::BlockDevice, drivers::block, fs::{vfs::{inode::InodeMode, Dentry, DentryInner, File, FileInner, Inode, InodeInner}, Kstat, OpenFlags, StatxTimestamp, Xstat, XstatMask, BLKSSZGET}, mm::UserPtrRaw, sync::mutex::SpinNoIrqLock, syscall::{SysError, SysResult}, task::current_task, utils::block_on};


pub struct LoopDevInode {
    inner: InodeInner,
    file: SpinNoIrqLock<Option<Arc<dyn File>>>,
    loop_info: SpinNoIrqLock<Option<LoopInfo>>,
    loop_info64: SpinNoIrqLock<Option<LoopInfo64>>,

}

impl LoopDevInode {
    pub fn new() -> Arc<Self> {
        let file = SpinNoIrqLock::new(None);
        let loop_info = SpinNoIrqLock::new(None);
        let loop_info64 = SpinNoIrqLock::new(None);
        Arc::new(Self {
            inner: InodeInner::new(None, InodeMode::BLOCK, 0),
            file,
            loop_info,
            loop_info64
        })
    }
    
    pub fn add_file(&self, file: Arc<dyn File>) -> Result<(), SysError> {
        *self.file.lock() = Some(file);
        Ok(())
    }

    pub fn clear(&self) -> Result<(), SysError> {
        if self.file.lock().is_none() {
            return Err(SysError::ENXIO)
        }
        *self.file.lock() = None;
        *self.loop_info.lock() = None;
        *self.loop_info64.lock() = None;
        Ok(())
    }

    pub fn file_size(&self) -> usize {
        if let Some(file) = self.file.lock().clone() {
            let file_size = file.size();
            self.inner.set_size(file_size);
            file_size
        } else {
            0
        }
    }
}


impl Inode for LoopDevInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn support_splice(&self) -> Result<(), SysError> {
        Err(SysError::EINVAL)
    }

    fn getattr(&self) -> Kstat {
        let size = self.file_size();
        let inner = self.inode_inner();
        Kstat {
            st_dev: 10, // random device id to bybass the mounted check
            st_ino: inner.ino as u64,
            st_mode: inner.mode().bits() as _,
            st_nlink: inner.nlink() as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 10,
            _pad0: 0,
            st_size: size as _,
            _pad1: 0,
            st_blksize: BLOCK_SIZE as _,
            st_blocks: (size / BLOCK_SIZE) as _,
            st_atime_sec: inner.atime().tv_sec as _,
            st_atime_nsec: inner.atime().tv_nsec as _,
            st_mtime_sec: inner.mtime().tv_sec as _,
            st_mtime_nsec: inner.mtime().tv_nsec as _,
            st_ctime_sec: inner.ctime().tv_sec as _,
            st_ctime_nsec: inner.ctime().tv_nsec as _,
        }
    }

    fn getxattr(&self, mask: XstatMask) -> Xstat {
        const SUPPORTED_MASK: XstatMask = XstatMask::from_bits_truncate({
            XstatMask::STATX_BLOCKS.bits |
            XstatMask::STATX_ATIME.bits |
            XstatMask::STATX_CTIME.bits |
            XstatMask::STATX_MTIME.bits |
            XstatMask::STATX_NLINK.bits |
            XstatMask::STATX_MODE.bits |
            XstatMask::STATX_SIZE.bits |
            XstatMask::STATX_INO.bits
        });
        let mask = mask & SUPPORTED_MASK;
        let size = self.file_size();
        let inner = self.inode_inner();
        Xstat {
            stx_mask: mask.bits,
            stx_blksize: 0,
            stx_attributes: 0,
            stx_nlink: inner.nlink() as u32,
            stx_uid: 0,
            stx_gid: 0,
            stx_mode: inner.mode().bits() as _,
            stx_ino: inner.ino as u64,
            stx_size: size as _,
            stx_blocks: 0,
            stx_attributes_mask: 0,
            stx_atime: StatxTimestamp {
                tv_sec: inner.atime().tv_sec as _,
                tv_nsec: inner.atime().tv_nsec as _,
            },
            stx_btime: StatxTimestamp {
                tv_sec: 0,
                tv_nsec: 0,
            },
            stx_ctime: StatxTimestamp {
                tv_sec: inner.ctime().tv_sec as _,
                tv_nsec: inner.ctime().tv_nsec as _,
            },
            stx_mtime: StatxTimestamp {
                tv_sec: inner.mtime().tv_sec as _,
                tv_nsec: inner.mtime().tv_nsec as _,
            },
            stx_rdev_major: 10,
            stx_rdev_minor: 10,
            stx_dev_major: 10,
            stx_dev_minor: 10,
            stx_mnt_id: 10,
            stx_dio_mem_align: 0,
            std_dio_offset_align: 0,
            stx_subvol: 0,
            stx_atomic_write_unit_min: 0,
            stx_atomic_write_unit_max: 0,
            stx_atomic_write_segments_max: 0,
            stx_dio_read_offset_align: 0,
        }
    }
}

pub struct LoopDevDentry {
    inner: DentryInner,
}

impl LoopDevDentry {
    pub fn new(
        name: &str,
        parent: Option<Arc<dyn Dentry>>
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: DentryInner::new(name, parent)
        })
    }
}

unsafe impl Send for LoopDevDentry {}
unsafe impl Sync for LoopDevDentry {}

impl Dentry for LoopDevDentry {
    fn dentry_inner(&self) -> &DentryInner {
        &self.inner
    }

    fn new(&self,
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, parent)
        });
        dentry
    }
    
    fn open(self: Arc<Self>, flags: OpenFlags) -> Option<Arc<dyn File>> {
        Some(LoopDevFile::new(self.clone()))
    }

    fn set_inode(&self, inode: Arc<dyn Inode>) {
        if self.inode().is_none() {
            *self.inner.inode.lock() = Some(inode);
        }
    }
}

pub struct LoopDevFile {
    inner: FileInner,

}

impl LoopDevFile {
    pub fn new(dentry: Arc<dyn Dentry>) -> Arc<Self> {
        let inner = FileInner {
            offset: 0.into(),
            dentry,
            flags: SpinNoIrqLock::new(OpenFlags::empty()),
        };
        
        Arc::new(Self { inner })
    }

    pub fn inode_dev(&self) -> Arc<LoopDevInode> {
        let inode = self.inode().unwrap();
        let loop_inode = inode.downcast_arc::<LoopDevInode>();
        match loop_inode {
            Ok(inode) => return inode,
            Err(_) => panic!()
        }
    }

    pub fn disk_file(&self) -> Arc<dyn File> {
        let inode = self.inode_dev();
        let file = inode.file.lock().clone().unwrap();
        file
    }
}

#[async_trait]
impl File for LoopDevFile {
    fn file_inner(&self) ->  &FileInner {
        &self.inner
    }

    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    async fn read(&self, buf: &mut [u8]) -> Result<usize, SysError> {
        let file = self.disk_file().clone();
        file.read(buf).await
    }

    async fn write(&self, buf: &[u8]) -> Result<usize, SysError> {
        let file = self.disk_file().clone();
        file.write(buf).await
    }

    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, SysError> {
        let file = self.disk_file().clone();
        file.read_at(offset, buf).await
    }

    async fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, SysError> {
        let file = self.disk_file().clone();
        file.write_at(offset, buf).await
    }
    

    fn ioctl(&self, cmd: usize, arg: usize) -> SysResult {
        let cmd = LoopIoctlCmd::from_repr(cmd)
            .ok_or(SysError::EINVAL)?;
        log::info!("[Loop] cmd {:?}, arg {:#x}", cmd, arg);
        let task = current_task().unwrap().clone();
        match cmd {
            LoopIoctlCmd::LOOP_GET_STATUS => {
                let status_ptr = UserPtrRaw::new(arg as *mut LoopInfo)
                    .ensure_write(&mut task.get_vm_space().lock())
                    .ok_or(SysError::EFAULT)?;
                let info = self.inode_dev().loop_info.lock().clone();
                match info {
                    Some(info) => status_ptr.write(info),
                    None => return Err(SysError::ENXIO),
                }
            }
            LoopIoctlCmd::LOOP_SET_FD => {
                let task = current_task().unwrap().clone();
                let file = task.with_fd_table(|t| t.get_file(arg))?;
                let _ = self.inode_dev().add_file(file);
            }
            LoopIoctlCmd::LOOP_CLR_FD => {
                let _ = self.inode_dev().clear()?;
            }
            LoopIoctlCmd::LOOP_SET_STATUS => {
                let status_ptr = UserPtrRaw::new(arg as *const LoopInfo)
                    .ensure_read(&mut task.get_vm_space().lock())
                    .ok_or(SysError::EFAULT)?;
                *self.inode_dev().loop_info.lock() = Some(*status_ptr.to_ref())
            }
            LoopIoctlCmd::LOOP_GET_STATUS64 => {
                let status_ptr = UserPtrRaw::new(arg as *mut LoopInfo64)
                    .ensure_write(&mut task.get_vm_space().lock())
                    .ok_or(SysError::EFAULT)?;
                let info = self.inode_dev().loop_info64.lock().clone();
                match info {
                    Some(info) => status_ptr.write(info),
                    None => return Err(SysError::ENXIO),
                }
            }
            LoopIoctlCmd::LOOP_SET_STATUS64 => {
                let status_ptr = UserPtrRaw::new(arg as *const LoopInfo64)
                    .ensure_read(&mut task.get_vm_space().lock())
                    .ok_or(SysError::EFAULT)?;
                *self.inode_dev().loop_info64.lock() = Some(*status_ptr.to_ref())
            }
            _ => todo!()
        }
        Ok(0)
    }
}

/// Defined in 
#[derive(FromRepr, Debug)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum LoopIoctlCmd {
    LOOP_SET_FD	= 0x4C00,
    LOOP_CLR_FD	= 0x4C01,
    LOOP_SET_STATUS	= 0x4C02,
    LOOP_GET_STATUS	= 0x4C03,
    LOOP_SET_STATUS64 = 0x4C04,
    LOOP_GET_STATUS64 = 0x4C05,
    LOOP_CHANGE_FD = 0x4C06,
    LOOP_SET_CAPACITY = 0x4C07,
    LOOP_SET_DIRECT_IO = 0x4C08,
    LOOP_SET_BLOCK_SIZE = 0x4C09,
    LOOP_CONFIGURE = 0x4C0A,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LoopInfo {
    number: i32,
    device: u32,
    inode: usize,
    rdevice: u32,
    roffset: i32,
    encrypt_type: i32,
    encrypt_key_size: i32,
    flags: i32,
    name: [u8; 64],        /* LO_NAME_SIZE = 64 */
    encrypt_key: [u8; 32], /* LO_KEY_SIZE = 32 */
    init: [usize; 2],
    reserved: [u8; 4],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LoopInfo64 {
    device: u64,
    inode: u64,
    rdev: u64,
    offset: u64,
    sizelimit: u64,
    number: u32,
    encrypt_type: u32,
    encrypt_key_size: u32,
    flags: u32,
    file_name: [u8; 64],        /* LO_NAME_SIZE = 64 */
    crypt_name: [u8; 64], /* LO_KEY_SIZE = 32 */
    crypt_key: [u8; 32],
    init: [u64; 2],
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum LoopFlags {
    LO_FLAGS_READ_ONLY	= 1,
	LO_FLAGS_AUTOCLEAR	= 4,
	LO_FLAGS_PARTSCAN	= 8,
	LO_FLAGS_DIRECT_IO	= 16,
}