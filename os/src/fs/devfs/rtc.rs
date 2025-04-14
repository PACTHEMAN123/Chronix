//! the real time clock device

use alloc::sync::Arc;
use async_trait::async_trait;
use alloc::boxed::Box;
use hal::instruction::{Instruction, InstructionHal};

use crate::{config::BLOCK_SIZE, fs::{vfs::{inode::InodeMode, Dentry, DentryInner, File, FileInner, Inode, InodeInner}, Kstat, OpenFlags, StatxTimestamp, SuperBlock, Xstat, XstatMask}, sync::mutex::SpinNoIrqLock, syscall::{SysError, SysResult}};


pub struct RtcFile {
    inner: FileInner,
}

impl RtcFile {
    pub fn new(dentry: Arc<dyn Dentry>) -> Arc<Self> {
        let inner = FileInner {
            offset: 0.into(),
            dentry,
            flags: SpinNoIrqLock::new(OpenFlags::empty()),
        };
        Arc::new(Self { inner })
    }
}

#[async_trait]
impl File for RtcFile {
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
        // reach EOF
        buf.fill(0);
        Ok(buf.len())
    }

    async fn write(&self, buf: &[u8]) -> Result<usize, SysError> {
        Ok(buf.len())
    }

    fn ioctl(&self, _cmd: usize, arg: usize) -> SysResult {
        unsafe {
            Instruction::set_sum();
            (arg as *mut RtcTime).write(RtcTime::default());
        }
        Ok(0)
    }
}

#[derive(Default)]
#[repr(C)]
pub struct RtcTime {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
}

pub struct RtcDentry {
    inner: DentryInner,
}

impl RtcDentry {
    pub fn new(
        name: &str,
        super_block: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: DentryInner::new(name, super_block, parent),
        })
    }
}

unsafe impl Send for RtcDentry {}
unsafe impl Sync for RtcDentry {}

impl Dentry for RtcDentry {
    fn dentry_inner(&self) -> &DentryInner {
        &self.inner
    }

    fn new(&self,
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, superblock, parent)
        });
        dentry
    }
    
    fn open(self: Arc<Self>, _flags: OpenFlags) -> Option<Arc<dyn File>> {
        Some(RtcFile::new(self.clone()))
    }
}

pub struct RtcInode {
    inner: InodeInner,
}

impl RtcInode {
    pub fn new(super_block: Arc<dyn SuperBlock>) -> Arc<Self> {
        let size = BLOCK_SIZE;
        Arc::new(Self {
            inner: InodeInner::new(super_block, InodeMode::FILE, size),
        })
    }
}

impl Inode for RtcInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn getattr(&self) -> crate::fs::Kstat {
        let inner = self.inode_inner();
        let len = inner.size();
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode.bits() as _,
            st_nlink: inner.nlink() as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: inner.size() as _,
            _pad1: 0,
            st_blksize: 512,
            st_blocks: (len / 512) as _,
            st_atime_sec: inner.atime().tv_sec as _,
            st_atime_nsec: inner.atime().tv_nsec as _,
            st_mtime_sec: inner.mtime().tv_sec as _,
            st_mtime_nsec: inner.mtime().tv_nsec as _,
            st_ctime_sec: inner.ctime().tv_sec as _,
            st_ctime_nsec: inner.ctime().tv_nsec as _,
        }
    }

    fn getxattr(&self, mask: crate::fs::XstatMask) -> crate::fs::Xstat {
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
        let inner = self.inode_inner();
        Xstat {
            stx_mask: mask.bits,
            stx_blksize: 0,
            stx_attributes: 0,
            stx_nlink: inner.nlink() as u32,
            stx_uid: 0,
            stx_gid: 0,
            stx_mode: inner.mode.bits() as _,
            stx_ino: inner.ino as u64,
            stx_size: inner.size() as _,
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
            stx_rdev_major: 0,
            stx_rdev_minor: 0,
            stx_dev_major: 0,
            stx_dev_minor: 0,
            stx_mnt_id: 0,
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