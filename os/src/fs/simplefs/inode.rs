use alloc::sync::Arc;

use crate::fs::{vfs::{inode::InodeMode, Inode, InodeInner}, Kstat, StatxTimestamp, SuperBlock, Xstat, XstatMask};



/// simple file system inode
/// it does not contain any resource
pub struct SpInode {
    inner: InodeInner,
}

impl SpInode {
    pub fn new(super_block: Arc<dyn SuperBlock>) -> Arc<Self> {
        let inner = InodeInner::new(super_block, InodeMode::DIR, 0);
        Arc::new(Self { inner })
    }
}

impl Inode for SpInode {
    fn inner(&self) -> &InodeInner {
        &self.inner
    }

    fn lookup(&self, _name: &str) -> Option<Arc<dyn Inode>> {
        None
    }

    fn getattr(&self) -> Kstat {
        let inner = self.inner();
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode.bits() as _,
            st_nlink: inner.nlink as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: 0,
            _pad1: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atime_sec: inner.atime.tv_sec as _,
            st_atime_nsec: inner.atime.tv_nsec as _,
            st_mtime_sec: inner.mtime.tv_sec as _,
            st_mtime_nsec: inner.mtime.tv_nsec as _,
            st_ctime_sec: inner.ctime.tv_sec as _,
            st_ctime_nsec: inner.ctime.tv_nsec as _,
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
        let inner = self.inner();
        Xstat {
            stx_mask: mask.bits,
            stx_blksize: 0,
            stx_attributes: 0,
            stx_nlink: inner.nlink as u32,
            stx_uid: 0,
            stx_gid: 0,
            stx_mode: inner.mode.bits() as _,
            stx_ino: inner.ino as u64,
            stx_size: inner.size as _,
            stx_blocks: 0,
            stx_attributes_mask: 0,
            stx_atime: StatxTimestamp {
                tv_sec: inner.atime.tv_sec as _,
                tv_nsec: inner.atime.tv_nsec as _,
            },
            stx_btime: StatxTimestamp {
                tv_sec: 0,
                tv_nsec: 0,
            },
            stx_ctime: StatxTimestamp {
                tv_sec: inner.ctime.tv_sec as _,
                tv_nsec: inner.ctime.tv_nsec as _,
            },
            stx_mtime: StatxTimestamp {
                tv_sec: inner.mtime.tv_sec as _,
                tv_nsec: inner.mtime.tv_nsec as _,
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