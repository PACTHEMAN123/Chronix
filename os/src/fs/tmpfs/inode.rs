//! inode in memory

use core::cmp;

use alloc::{string::{String, ToString}, sync::{Arc, Weak}};

use crate::{config::{BLOCK_SIZE, PAGE_SIZE}, fs::{page::{cache::PageCache, page::Page}, vfs::{inode::InodeMode, Inode, InodeInner}, Kstat, StatxTimestamp, SuperBlock, Xstat, XstatMask}, sync::mutex::SpinNoIrqLock, syscall::SysError};

pub struct TmpInode {
    inner: InodeInner,
    cache: Arc<PageCache>,
    symlink_path: SpinNoIrqLock<String>,
}

unsafe impl Send for TmpInode {}
unsafe impl Sync for TmpInode {}

impl TmpInode {
    /// create a new tmp inode
    pub fn new(super_block: Weak<dyn SuperBlock>, mode: InodeMode) -> Arc<Self> {
        let inner = InodeInner::new(Some(super_block), mode, 0);
        let cache = Arc::new(PageCache::new());
        let symlink_path = SpinNoIrqLock::new(String::new());
        Arc::new(Self { inner, cache, symlink_path })
    }
}

impl Inode for TmpInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn cache(&self) -> Option<Arc<PageCache>> {
        Some(self.cache.clone())
    }

    fn read_page_at(self: Arc<Self>, offset: usize) -> Option<Arc<Page>> {
        let size = self.inner.size();
        if offset >= size {
            log::debug!("[Tmp Inode]: read_page_at: reach EOF, offset: {} size: {}", offset, size);
            return None;
        }
        let page_cache = self.cache().unwrap();
        // since tmp file relies only on page cache
        // if not found, may indicates that a "hole" is in the page cache
        // due to the un-continuous write
        // return a page filled with zero
        let page = if let Some(page) = page_cache.get_page(offset) {
            page.clone()
        } else {
            let page = Page::new(offset);
            page_cache.insert_page(offset, page.clone());
            page_cache.update_end(offset + PAGE_SIZE);
            page
        };
        Some(page)
    }

    fn cache_read_at(self: Arc<Self>, offset: usize, buf: &mut [u8]) -> Result<usize, i32> {
        let size = self.inner.size();
        log::info!("cur size: {}, buf size: {}", size, buf.len());
        // if offset >= size {
        //     log::debug!("[Tmp Inode]: read_page_at: reach EOF, offset: {} size: {}", offset, size);
        //     return Ok(0);
        // }
        let mut total_read_size = 0usize;
        let mut current_offset = offset;
        let mut buf_offset = 0usize;
        while buf_offset < buf.len() {
            let cache = self.cache.clone();
            if current_offset >= cache.end() {
                break;
            }
            let page_offset = current_offset / PAGE_SIZE * PAGE_SIZE;
            let in_page_offset = current_offset % PAGE_SIZE;
            let page = if let Some(page) = cache.get_page(page_offset) {
                page.clone()
            } else {
                let page = Page::new(page_offset);
                cache.insert_page(page_offset, page.clone());
                // cache.update_end(page_offset + PAGE_SIZE);
                page
            };
            let buf_read_size = cmp::min(cache.end() - current_offset, buf.len() - buf_offset);
            let page_read_size = page.read_at(in_page_offset, &mut buf[buf_offset..buf_offset + buf_read_size]);
            // should truncate the read size if larger than file size
            // if current_offset + page_read_size > size {
            //     assert!(size >= current_offset);
            //     total_read_size += size - current_offset;
            //     break;
            // }
            total_read_size += page_read_size;
            buf_offset += page_read_size;
            current_offset += page_read_size; 
        }
        Ok(total_read_size)
    }

    fn cache_write_at(self: Arc<Self>, offset: usize, buf: &[u8]) -> Result<usize, i32> {
        let mut total_write_size = 0usize;
        let mut current_offset = offset;
        let mut buf_offset = 0usize;
        let cache = self.cache.clone();

        while buf_offset < buf.len() {
            let page_offset = current_offset / PAGE_SIZE * PAGE_SIZE;
            let in_page_offset = current_offset % PAGE_SIZE;

            let page = if let Some(page) = cache.get_page(page_offset) {
                page.clone()
            } else {
                let page = Page::new(page_offset);
                cache.insert_page(page_offset, page.clone());
                page
            };
            let page_write_size = page.write_at(in_page_offset, &buf[buf_offset..]);
            page.set_dirty();
            cache.update_end(page_offset + page_write_size + in_page_offset);
            self.inner.set_size(cache.end());

            total_write_size += page_write_size;
            buf_offset += page_write_size;
            current_offset += page_write_size;
        }

        Ok(total_write_size)
    }

    fn create(&self, _name: &str, mode: InodeMode) -> Result<Arc<dyn Inode>, SysError> {
        let sb = self.inode_inner().super_block.clone().unwrap();
        Ok(TmpInode::new(sb, mode))
    }

    fn remove(&self, _name: &str, _mode: InodeMode) -> Result<usize, i32> {
        // do nothing
        // when call unlink, the dentry will drop inode, becoming a neg dentry
        Ok(0)
    }

    fn truncate(&self, size: usize) -> Result<usize, SysError> {
        let old_size = self.inner.size();
        if size > old_size {
            // expand the page cache
            let page_cache = self.cache.clone();
            let offset_aligned_start = old_size / PAGE_SIZE * PAGE_SIZE;
            for offset_aligned in (offset_aligned_start..size).step_by(PAGE_SIZE) {
                let page = Page::new(offset_aligned);
                page_cache.insert_page(offset_aligned, page.clone());
            }
            self.inner.set_size(size);
            Ok(size)
        } else if old_size == size {
            return Ok(size)
        } else {
            log::warn!("not support reduce size for tmp file");
            self.inner.set_size(size);
            return Ok(size)
        }
    }

    fn getattr(&self) -> Kstat {
        let inner = self.inode_inner();
        let size = inner.size();
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode().bits() as _,
            st_nlink: inner.nlink() as u32,
            st_uid: inner.uid(),
            st_gid: inner.gid(),
            st_rdev: 0,
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
        let size = inner.size();
        Xstat {
            stx_mask: mask.bits,
            stx_blksize: BLOCK_SIZE as _,
            stx_attributes: 0,
            stx_nlink: inner.nlink() as u32,
            stx_uid: inner.uid(),
            stx_gid: inner.gid(),
            stx_mode: inner.mode().bits() as _,
            stx_ino: inner.ino as u64,
            stx_size: size as _,
            stx_blocks: (size / BLOCK_SIZE) as _,
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

    // call by link_path inode parent
    fn symlink(&self, target_path: &str, _link_path: &str) -> Result<Arc<dyn Inode>, SysError> {
        let sb = self.inode_inner().super_block.clone().unwrap();
        let inode = TmpInode::new(sb, InodeMode::LINK);
        inode.symlink_path.lock().push_str(target_path);
        Ok(inode)
    }

    fn readlink(&self) -> Result<String, SysError> {
        assert_eq!(self.inode_type(), InodeMode::LINK);
        Ok(self.symlink_path.lock().to_string())
    }

    fn link(&self, _target: &str) -> Result<usize, SysError> {
        // link at dentry layer
        let old_link_nums = self.inode_inner().nlink();
        self.inode_inner().set_nlink(old_link_nums + 1);
        Ok(0)
    }
}

pub trait InodeContent {
    fn serialize(&self) -> String;
}

pub struct TmpSysInode {
    inner: InodeInner,
    content: Arc<dyn InodeContent>,
}

impl TmpSysInode {
    pub fn new(mode: InodeMode, content: Arc<dyn InodeContent>) -> Arc<Self> {
        let inner = InodeInner::new(
            None, 
            mode, 
            content.serialize().len()
        );
        Arc::new(Self { inner, content })
    }
}

impl Inode for TmpSysInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32> {
        let content = self.content.serialize();
        let size = content.len();
        self.inner.set_size(size);
        if offset > size {
            return Ok(0)
        }
        let read_size = cmp::min(buf.len(), size - offset);
        buf[..read_size].copy_from_slice(&content.as_bytes()[offset..offset+read_size]);
        Ok(read_size)
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize, i32> {
        Ok(0)
    }

    fn getattr(&self) -> Kstat {
        let size = self.content.serialize().len();
        self.inode_inner().set_size(size);
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
        let size = self.content.serialize().len();
        self.inode_inner().set_size(size);

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
}