

use alloc::{string::ToString, sync::Arc, vec::Vec};
use alloc::string::String;

use crate::fs::tmpfs::file::TmpFile;
use crate::fs::vfs::Inode;
use crate::fs::{Kstat, StatxTimestamp, Xstat, XstatMask};
use crate::{fs::{tmpfs::dentry::TmpDentry, vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, InodeInner}}, syscall::SysError, task::current_task};


pub struct FdDentry {
    inner: DentryInner,
}

unsafe impl Send for FdDentry {}
unsafe impl Sync for FdDentry {}

impl FdDentry {
    pub fn new(
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, parent)
        });
        dentry
    }
}

impl Dentry for FdDentry {
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

    fn open(self: Arc<Self>, _flags: crate::fs::OpenFlags) -> Option<Arc<dyn crate::fs::vfs::File>> {
        Some(TmpFile::new_arc(self.clone()))
    }

    fn load_child_dentry(self: Arc<Self>) -> Result<Vec<Arc<dyn Dentry>>, SysError> {
        let mut child_dentrys: Vec<Arc<dyn Dentry>> = Vec::new();
        let task = current_task().unwrap().clone();
        task.with_fd_table(|t| {
            let fds = &t.fd_table;
            for i in 0..fds.len() {
                if let Some(fd_info) = &fds[i] {
                    let name = i.to_string();
                    let child = TmpDentry::new(&name, None);
                    let path = fd_info.file.file_inner().dentry.path();
                    let fd_inode = FdChildInode::new(&path);
                    child.set_inode(fd_inode);
                    child_dentrys.push(child);
                }
            }
        });
        Ok(child_dentrys)
    }

    fn get_child(&self, name: &str) -> Option<Arc<dyn Dentry>> {
        let task = current_task().unwrap().clone();
        task.with_fd_table(|t| {
            let fds = &t.fd_table;
            for i in 0..fds.len() {
                if let Some(fd_info) = &fds[i] {
                    let fd_name = i.to_string();
                    if fd_name == name {
                        let child = TmpDentry::new(&name, None);
                        let path = fd_info.file.file_inner().dentry.path();
                        let fd_inode = FdChildInode::new(&path);
                        child.set_inode(fd_inode);
                        return Some(child)
                    }
                }
            }
            None
        })
    }
}

pub struct FdChildInode {
    inner: InodeInner,
    link_path: String,
}

impl FdChildInode {
    pub fn new(file_path: &str) -> Arc<Self> {
        let inner = InodeInner::new(None, InodeMode::LINK, 0);
        Arc::new(Self {
            inner: inner,
            link_path: file_path.to_string()
        })
    }
}

impl Inode for FdChildInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn getattr(&self) -> crate::fs::Kstat {
        let inner = self.inode_inner();
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode().bits() as _,
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
            stx_mode: inner.mode().bits() as _,
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

    fn readlink(&self) -> Result<String, SysError> {
        Ok(self.link_path.clone())
    }
}





