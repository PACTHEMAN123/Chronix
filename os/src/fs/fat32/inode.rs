//! fat32 inode implement for vfs
//! fat only have file and dir, they will be view same as Inode in VFS

use core::cmp;

use alloc::string::String;
use alloc::{vec, vec::Vec};
use alloc::sync::Arc;
use fatfs::{Error, Dir, File, LossyOemCpConverter, NullTimeProvider};
use fatfs::{Read, Seek, SeekFrom, Write};
use log::{debug, info};

use crate::fs::vfs::inode::InodeMode;
use crate::fs::{Kstat, SuperBlock};
use crate::{fs::vfs::{Inode, InodeInner}, sync::UPSafeCell};

use super::disk::DiskCursor;
use super::superblock::FatSuperBlock;

/// fit fat file into inode
pub struct FatFileInode {
    inner: InodeInner,
    file: UPSafeCell<FatFileMeta>,
}

pub struct FatFileMeta {
    #[allow(unused)]
    pub(crate) name: String,
    pub(crate) inner: File<'static, DiskCursor, NullTimeProvider, LossyOemCpConverter>,
    pub(crate) size: usize,
}

/// fit fat dir into inode
pub struct FatDirInode {
    pub(crate) inner: InodeInner,
    pub(crate) dir: UPSafeCell<FatDirMeta>,
}

pub struct FatDirMeta {
    #[allow(unused)]
    pub(crate) name: String,
    pub(crate) inner: Dir<'static, DiskCursor, NullTimeProvider, LossyOemCpConverter>,
    pub(crate) size: usize,
}

impl Inode for FatFileInode {
    fn inner(&self) -> &InodeInner {
        &self.inner
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32> {
        //info!("try to read at: offset: {}, buf len: {}", offset, buf.len());
        let inner = self.file.exclusive_access();

        if offset >= inner.size {
            return Ok(0);
        }
        let seek_curr = SeekFrom::Start(offset as _);
        inner.inner.seek(seek_curr).expect("seek failed");
        let len = inner.size;
        debug!("off: {:#x} rlen: {:#x}", offset, len);
        // read cached file.
        inner
            .inner
            .seek(SeekFrom::Start(offset as u64))
            .expect("seek failed");
        let rlen = cmp::min(buf.len(), len as usize - offset);
        inner
            .inner
            .read_exact(&mut buf[..rlen])
            .expect("read failed");
        Ok(rlen)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, i32> {
        let inner = self.file.exclusive_access();

        // if offset > len
        let seek_curr = SeekFrom::Start(offset as _);
        let curr_off = inner.inner.seek(seek_curr).expect("seek failed") as usize;
        if offset != curr_off {
            let buffer = vec![0u8; 512];
            loop {
                let wlen = cmp::min(offset - inner.size, 512);

                if wlen == 0 {
                    break;
                }
                let real_wlen = inner.inner.write(&buffer).expect("write failed");
                inner.size += real_wlen;
            }
        }

        inner.inner.write_all(buf).expect("write failed");

        if offset + buf.len() > inner.size {
            inner.size = offset + buf.len();
        }
        Ok(buf.len())
    }

    fn truncate(&self, size: u64) -> Result<usize, i32> {
        self.file
            .exclusive_access()
            .inner
            .seek(SeekFrom::Start(size as u64))
            .expect("seek failed");
        self.file.exclusive_access().inner.truncate().expect("truncate failed");
        Ok(0)
    }

    fn getattr(&self) -> crate::fs::Kstat {
        Kstat {
            st_ino: 1,
            st_mode: InodeMode::FILE.bits(),
            st_atime_sec: 0,
            st_atime_nsec: 0,
            st_blksize: 512,
            st_ctime_sec: 0,
            st_ctime_nsec: 0,
            st_blocks: self.file.exclusive_access().size as i64 / 512,
            st_dev: 0,
            st_gid: 0,
            st_mtime_sec: 0,
            st_mtime_nsec: 0,
            st_nlink: 1,
            st_size: self.file.exclusive_access().size as i64,
            st_rdev: 0,
            st_uid: 0,
            _pad0: 0,
            _pad1: 0,
        }
    }

    fn lookup(&self, _name: &str) -> Option<Arc<dyn Inode>> {
        panic!("fat32 file inode dont support lookup!")
    }

    fn ls(&self) -> Vec<String> {
        panic!("fat32 file inode dont support ls!")
    }

    fn unlink(&self) -> Result<usize, i32> {
        panic!("fat32 file can only be unlink by parent dir")
    }

    fn create(&self, _path: &str, _mode: InodeMode) -> Option<Arc<dyn Inode>> {
        panic!("fat32 file can not create file!")
    }

    fn cache_read_at(self: Arc<Self>, _offset: usize, _buf: &mut [u8]) -> Result<usize, i32> {
        panic!("not support cached read")
    }

    fn cache_write_at(self: Arc<Self>, _offset: usize, _buf: &[u8]) -> Result<usize, i32> {
        panic!("not support cached write")
    }

    fn remove(&self, _name: &str, _mode: InodeMode) -> Result<usize, i32> {
        panic!()
    }
}

impl Inode for FatDirInode {
    fn inner(&self) -> &InodeInner {
        &self.inner
    }
    fn create(&self, name: &str, mode: InodeMode) -> Option<Arc<dyn Inode>> {
        let dir = self.dir.exclusive_access();
        let super_block = self.inner().super_block.upgrade()?.clone();
        match mode {
            InodeMode::FILE => {
                dir.inner
                .create_file(name)
                .map(|file| -> Option<Arc<dyn Inode>> {
                    Some(
                        Arc::new(FatFileInode {
                            inner: InodeInner::new(super_block, mode, 0),
                            file: UPSafeCell::new(FatFileMeta {
                                name: String::from(name),
                                inner: file,
                                size: 0,
                            })
                        }))
                })
                .expect("create file failed")
            }
            InodeMode::DIR => {
                dir.inner
                .create_dir(name)
                .map(|dir| -> Option<Arc<dyn Inode>> {
                    Some(
                        Arc::new(FatDirInode {
                            inner: InodeInner::new(super_block, mode, 0),
                            dir: UPSafeCell::new(FatDirMeta {
                                name: String::from(name),
                                inner: dir,
                                size: 0,
                            })
                        }))
                })
                .expect("create dir failed")
            }
            _ => {
                panic!("fat32 not support!")
            }
        }
    }
    fn getattr(&self) -> crate::fs::Kstat {
        Kstat {
            st_ino: 1,
            st_mode: InodeMode::DIR.bits(),
            st_atime_sec: 0,
            st_atime_nsec: 0,
            st_blksize: 512,
            st_ctime_sec: 0,
            st_ctime_nsec: 0,
            st_blocks: self.dir.exclusive_access().size as i64 / 512,
            st_dev: 0,
            st_gid: 0,
            st_mtime_sec: 0,
            st_mtime_nsec: 0,
            st_nlink: 1,
            st_size: self.dir.exclusive_access().size as i64,
            st_rdev: 0,
            st_uid: 0,
            _pad0: 0,
            _pad1: 0,
        }
    }
    fn lookup(&self, name: &str) -> Option<Arc<dyn Inode>> {
        let dir = self.dir.exclusive_access();
        let target = dir.inner
        .iter()
        .find(|x| {
            x.as_ref().unwrap().file_name() == name
        });
        if target.is_none() {
            return None;
        }
        let target = target.unwrap().unwrap();
        if target.is_dir() {
            Some(Arc::new(FatDirInode {
                inner: InodeInner::new(
                self.inner().super_block.upgrade()?.clone(),
                InodeMode::DIR,
                0,
                ),
                dir: UPSafeCell::new(FatDirMeta {
                    name: String::from(name),
                    inner: target.to_dir(),
                    size: 0,
                }),
            }))
        } else if target.is_file() {
            Some(Arc::new(FatFileInode {
                inner: InodeInner::new(
                    self.inner().super_block.upgrade()?.clone(),
                InodeMode::FILE,
                0,
                ),
                file: UPSafeCell::new(FatFileMeta {
                    name: String::from(name),
                    inner: target.to_file(),
                    size: target.len() as usize,
                }),
            }))
        } else {
            panic!("should not reach here!")
        }
    }

    fn ls(&self) -> Vec<String> {
        let dir = self.dir.exclusive_access();
        dir.inner
        .iter()
        .filter_map(|x| {
            let x = x.unwrap();
            if x.file_name() == "." || x.file_name() == ".." {
                return None;
            }
            Some(x.file_name())
        })
        .collect()
    }

    fn unlink(&self) -> Result<usize, i32> {
        panic!("fat32 not support for unlink")
    }

    fn remove(&self, name: &str, _mode: InodeMode) -> Result<usize, i32> {
        let _ = self.dir.exclusive_access().inner.remove(name);
        Ok(0)
    }

    fn cache_read_at(self: Arc<Self>, _offset: usize, _buf: &mut [u8]) -> Result<usize, i32> {
        panic!()
    }

    fn cache_write_at(self: Arc<Self>, _offset: usize, _buf: &[u8]) -> Result<usize, i32> {
        panic!()
    }

    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> Result<usize, i32> {
        panic!()
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize, i32> {
        panic!()
    }

    fn truncate(&self, _size: u64) -> Result<usize, i32> {
        panic!()
    }
}

