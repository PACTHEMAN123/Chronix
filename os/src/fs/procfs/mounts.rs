//! /proc/mounts file

use alloc::{string::{String, ToString}, sync::{Arc, Weak}};
use async_trait::async_trait;
use alloc::boxed::Box;

use crate::{config::BLOCK_SIZE, fs::{vfs::{inode::InodeMode, Dentry, DentryInner, File, FileInner, Inode, InodeInner}, Kstat, OpenFlags, StatxTimestamp, SuperBlock, Xstat, XstatMask, FS_MANAGER}, sync::mutex::SpinNoIrqLock, syscall::SysError};


pub struct MountsFile {
    inner: FileInner,
}

impl MountsFile {
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
impl File for MountsFile {
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
        let info = list_mounts();
        let len = info.len();
        let pos = self.pos();
        if self.pos() >= len {
            return Ok(0);
        }
        buf[..len].copy_from_slice(info.as_bytes());
        self.set_pos(pos + len);
        Ok(len)
    }

    async fn write(&self, _buf: &[u8]) -> Result<usize, SysError> {
        Ok(0)
    }
}

pub struct MountsDentry {
    inner: DentryInner,
}

impl MountsDentry {
    pub fn new(
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: DentryInner::new(name, parent),
        })
    }
}

unsafe impl Send for MountsDentry {}
unsafe impl Sync for MountsDentry {}

impl Dentry for MountsDentry {
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
    
    fn open(self: Arc<Self>, _flags: OpenFlags) -> Option<Arc<dyn File>> {
        Some(MountsFile::new(self.clone()))
    }
}

pub struct MountsInode {
    inner: InodeInner,
}

impl MountsInode {
    pub fn new(super_block: Weak<dyn SuperBlock>) -> Arc<Self> {
        let size = BLOCK_SIZE;
        Arc::new(Self {
            inner: InodeInner::new(Some(super_block), InodeMode::FILE, size),
        })
    }
}

impl Inode for MountsInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn getattr(&self) -> crate::fs::Kstat {
        let inner = self.inode_inner();
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
            st_blksize: 0,
            st_blocks: 0,
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

pub fn list_mounts() -> String {
    let mut res = "".to_string();
    let fs_manager = FS_MANAGER.lock();
    for (_, fs) in fs_manager.iter() {
        let sbs = fs.inner().supers.lock();
        for (mount_path, _) in sbs.iter() {
            // device name: (todo)
            res += "device";
            res += " ";
            // mount point
            res += mount_path;
            res += " ";
            // fs type name
            res += fs.name();
            res += " ";
            // fs stat flags (todo)
            res += "rw,nosuid,nodev,noexec,relatime";
            res += " ";
            
            res += "0 0\n";
        }
    }
    res
}