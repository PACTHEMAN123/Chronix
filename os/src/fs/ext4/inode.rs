//! implement the vfs operations and node operations for ext4 filesystem
//! definition in `vfs.rs`

use core::cell::RefCell;
use core::cmp;
use core::ptr::NonNull;

use alloc::string::{String, ToString};
use alloc::ffi::CString;
use hal::addr::RangePPNHal;
use super::disk::Disk;
use alloc::sync::{Arc, Weak};
use alloc::{vec, vec::Vec};

use log::*;
use crate::fs::page::cache::PageCache;
use crate::fs::page::page::{Page, PAGE_SIZE};
use crate::fs::vfs::inode::InodeMode;
use crate::fs::vfs::{InodeInner, Inode};
use crate::fs::{Kstat, StatxTimestamp, SuperBlock, Xstat, XstatMask};
use crate::sync::mutex::SpinNoIrqLock;
use crate::sync::UPSafeCell;
use crate::utils::rel_path_to_abs;
use crate::syscall::{SysError, SysResult};

use lwext4_rust::bindings::{
    EXT4_DE_SYMLINK, O_APPEND, O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY, SEEK_CUR, SEEK_END, SEEK_SET
};
use lwext4_rust::{Ext4BlockWrapper, Ext4File, InodeTypes, KernelDevOp};

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};

use crate::config::BLOCK_SIZE;

/// The inode of the Ext4 filesystem
pub struct Ext4Inode {
    inner: InodeInner,
    file: SpinNoIrqLock<Ext4File>,
    cache: Arc<PageCache>,
}

unsafe impl Send for Ext4Inode {}
unsafe impl Sync for Ext4Inode {}

impl Ext4Inode {
    /// Create a new inode
    pub fn new(super_block: Weak<dyn SuperBlock>, path: &str, types: InodeTypes) -> Self {
        //info!("Inode new {:?} {}", types, path);
        let mode = InodeMode::from_inode_type(types.clone());
        let mut file  = Ext4File::new(path, types);
        // (todo) notice that lwext4 mention in file_size(): should open file as RDONLY first 
        // may be a bug in the future
        let size = file.file_size();
        Self {
            inner: InodeInner::new(Some(super_block.clone()), mode, size as usize),
            file: SpinNoIrqLock::new(file),
            cache: Arc::new(PageCache::new()),
        }
    }

    #[allow(unused)]
    fn path_deal_with(&self, path: &str) -> String {
        if path.starts_with('/') {
            warn!("path_deal_with: {}", path);
        }
        let p = path.trim_matches('/'); // 首尾去除
        if p.is_empty() || p == "." {
            return String::new();
        }

        if let Some(rest) = p.strip_prefix("./") {
            //if starts with "./"
            return self.path_deal_with(rest);
        }
        let rest_p = p.replace("//", "/");
        if p != rest_p {
            return self.path_deal_with(&rest_p);
        }

        //Todo ? ../
        //注：lwext4创建文件必须提供文件path的绝对路径
        let file = self.file.lock();
        let path = file.get_path();
        let fpath = String::from(path.to_str().unwrap().trim_end_matches('/')) + "/" + p;
        info!("dealt with full path: {}", fpath.as_str());
        fpath
    }
}

impl Inode for Ext4Inode {

    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn cache(&self) -> Option<Arc<PageCache>> {
        Some(self.cache.clone())
    }

    fn read_page_at(self: Arc<Self>, offset: usize) -> Option<Arc<Page>> {
        let size = self.getattr().st_size as usize;
        if offset >= size {
            info!("[Ext4 INode]: read_page_at: reach EOF, offset: {} size: {}", offset, size);
            return None;
        }
        let page_cache = self.cache().unwrap();
        let page = if let Some(page) = page_cache.get_page(offset) {
            page.clone()
        } else {
            let mut page = Page::new(offset);
            let read_size = Arc::get_mut(&mut page).unwrap().read_from(self.clone(), offset);
            page_cache.insert_page(offset, page.clone());
            page_cache.update_end(offset + read_size);
            page
        };
        Some(page)
    }

    /// Look up the node with given `name` in the directory
    /// Return the node if found.
    fn lookup(&self, name: &str) -> Option<Arc<dyn Inode>> {
        let mut file = self.file.lock();
        let full_path = String::from(file.get_path().to_str().unwrap().trim_end_matches('/')) + "/" + name;
        log::debug!("try to look up {}", full_path);
        if file.check_inode_exist(full_path.as_str(), InodeTypes::EXT4_DE_REG_FILE) {
            log::debug!("lookup {} success", name);
            return Some(Arc::new(Ext4Inode::new(
                self.inode_inner().super_block.clone().unwrap(), 
                full_path.as_str(), 
                InodeTypes::EXT4_DE_REG_FILE)));
        } else if file.check_inode_exist(full_path.as_str(), InodeTypes::EXT4_DE_DIR) {
            log::debug!("lookup dir {} success", name);
            return Some(Arc::new(Ext4Inode::new(
                self.inode_inner().super_block.clone().unwrap(), 
                full_path.as_str(), 
                InodeTypes::EXT4_DE_DIR)));
        } else if file.check_inode_exist(full_path.as_str(), InodeTypes::EXT4_DE_SYMLINK) {
            log::debug!("look up symlink {} success", name);
            return Some(Arc::new(Ext4Inode::new(
                self.inode_inner().super_block.clone().unwrap(),
                full_path.as_str(),
                InodeTypes::EXT4_DE_SYMLINK)));
        }
        //info!("lookup {} failed", name);
        None
    }

    /// list all files' name in the directory
    fn ls(&self) -> Vec<String> {
        let file = self.file.lock();

        if file.get_type() != InodeTypes::EXT4_DE_DIR {
            info!("not a directory");
        }

        let (name, inode_type) = match file.lwext4_dir_entries() {
            Ok((name, inode_type)) => (name, inode_type),
            Err(e) => {
                panic!("error when ls: {}", e);
            }
        };
        let mut name_iter = name.iter();
        let  _inode_type_iter = inode_type.iter();

        let mut names = Vec::new();
        while let Some(iname) = name_iter.next() {
            // notice that the name from lwext4_dir_entries, are C string end with '\0'
            // in order to make ls compatable with other parts, we should remove the '\0'
            let cname = String::from(core::str::from_utf8(iname).unwrap());
            let name = cname.trim_end_matches('\0').to_string();
            names.push(name);
        }
        names
    }

    /// Read data from inode at offset
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32> {
        debug!("To read_at {}, buf len={}", offset, buf.len());
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY)?;

        file.file_seek(offset as i64, SEEK_SET)?;
        let r = file.file_read(buf);

        let _ = file.file_close();
        r
    }

    /// Write data to inode at offset
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, i32> {
        debug!("To write_at {}, buf len={}", offset, buf.len());
        let mut file =  self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR)?;

        file.file_seek(offset as i64, SEEK_SET)?;
        let r = file.file_write(buf);

        let _ = file.file_close();
        r
    }

    fn cache_read_at(self: Arc<Self>, offset: usize, buf: &mut [u8]) -> Result<usize, i32> {
        // get the page-aligned offset
        let mut total_read_size = 0usize;
        let mut current_offset = offset;
        let mut buf_offset = 0usize;

        // get the file size on disk (may not sync)
        let file_size = {
            let mut file = self.file.lock();
            let cpath = file.get_path();
            let path = cpath.to_str().unwrap();
            file.file_open(path, O_RDWR)?;
            file.file_size() as usize
        };

        while buf_offset < buf.len() {
            let cache = self.cache.clone();
            let max_end = cmp::max(cache.end(), file_size);
            // info!("current offset: {:#x}, file end: {:#x}", current_offset, max_end);
            if current_offset >= max_end {
                break;
            }
            let page_offset = current_offset / PAGE_SIZE * PAGE_SIZE;
            let in_page_offset = current_offset % PAGE_SIZE;

            // get the cached page or read page using IO and store in cache
            
            let page = if let Some(page) = cache.get_page(page_offset) {
                // info!("[PAGE CACHE]: read hit at offset: {:#x}", page_offset);
                page.clone()
            } else {
                // info!("[PAGE CACHE]: read miss at offset: {:#x}", page_offset);
                // direct read at the offset of page size
                let mut page = Page::new(page_offset);
                let read_size = Arc::get_mut(&mut page).unwrap()
                    .read_from(self.clone(), page_offset);
                cache.insert_page(page_offset, page.clone());
                cache.update_end(page_offset + read_size);
                page
            };

            // now use the page to fill in the buf
            let buf_read_size = cmp::min(max_end - current_offset, buf.len() - buf_offset);
            let page_read_size = page.read_at(in_page_offset, &mut buf[buf_offset..buf_offset + buf_read_size]);
            //info!("read at offset: {}, read_size: {}", in_page_offset, page_read_size);

            total_read_size += page_read_size;
            buf_offset += page_read_size;
            current_offset += page_read_size;
        }

        // log::info!("[cache_read_at] buf len {}, file offset {:#x}, read size {:#x}", buf.len(), offset ,total_read_size);
        Ok(total_read_size)
    }

    fn cache_write_at(self: Arc<Self>, offset: usize, buf: &[u8]) -> Result<usize, i32> {
        // get file size
        let file_size = {
            let mut file = self.file.lock();
            let cpath = file.get_path();
            let path = cpath.to_str().unwrap();
            file.file_open(path, O_RDWR)?;
            file.file_size() as usize
        };
        // get the page-aligned offset
        let mut total_write_size = 0usize;
        let mut current_offset = offset;
        let mut buf_offset = 0usize;

        let cache = self.cache.clone();

        while buf_offset < buf.len() {
            let page_offset = current_offset / PAGE_SIZE * PAGE_SIZE;
            let in_page_offset = current_offset % PAGE_SIZE;

            // get the cached page or read page using IO and store in cache
            let page = if let Some(page) = cache.get_page(page_offset) {
                // info!("[PAGE CACHE]: read hit at offset: {:#x}", page_offset);
                page.clone()
            } else {
                // info!("[PAGE CACHE]: read miss at offset: {:#x}", page_offset);
                let mut page = Page::new(page_offset);
                if page_offset < file_size {
                    // write inside the file bound, should read out the data first
                    let _ = Arc::get_mut(&mut page).unwrap().read_from(self.clone(), page_offset);
                }
                cache.insert_page(page_offset, page.clone());
                page
            };

            // now use the buf to fill in the page
            let page_write_size = page.write_at(in_page_offset, &buf[buf_offset..]);
            page.set_dirty();
            cache.update_end(page_offset + page_write_size + in_page_offset);

            total_write_size += page_write_size;
            buf_offset += page_write_size;
            current_offset += page_write_size;
        }

        // log::info!("[cache_write_at] buf len {}, offset {:#x}, write size {:#x}", buf.len(), offset, total_write_size);
        Ok(total_write_size)
    }

    /// Truncate the inode to the given size
    fn truncate(&self, size: usize) -> Result<usize, SysError> {
        log::info!("truncate file to size {}", size);
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR).expect("file open failed");
        let t = file.file_truncate(size as _).map_err(|e| SysError::from_i32(e))?;
        let _ = file.file_close();
        Ok(t)
    }

    /// Create a new inode and return the inode
    fn create(&self, name: &str, mode: InodeMode) -> Result<Arc<dyn Inode>, SysError> {
        let ty: InodeTypes = mode.get_type().into();
        let mut file = self.file.lock();
        let parent_path = file.get_path().to_str().expect("cpath failed").to_string();
        let fpath = rel_path_to_abs(&parent_path, name).unwrap();
        info!("create {:?} on Ext4fs: {}", ty, fpath);
        //let fpath = self.path_deal_with(&fpath);
        let fpath = fpath.as_str();
        if fpath.is_empty() {
            info!("given path is empty");
            return Err(SysError::EINVAL);
        }

        let types = ty;

        let result = if file.check_inode_exist(fpath, types.clone()) {
            info!("inode already exists");
            return Err(SysError::EEXIST)
        } else {
            if types == InodeTypes::EXT4_DE_DIR {
                file.dir_mk(fpath)
            } else {
                file.file_open(fpath, O_WRONLY | O_CREAT | O_TRUNC)
                    .expect("create file failed");
                file.file_close()
            }
        };

        match result {
            Err(e) => {
                error!("create inode failed: {}", e);
                return Err(SysError::from_i32(e));
            }
            Ok(_) => {
                info!("create inode success");
                Ok(Arc::new(Ext4Inode::new(
                    self.inode_inner().super_block.clone().unwrap(),
                    fpath, types)))
            }
        }
    }

    fn getattr(&self) -> Kstat {
        let inner = self.inode_inner();
        let mut file = self.file.lock();
        let ty = file.get_type();

        let size = if ty == InodeTypes::EXT4_DE_REG_FILE {
            let path = file.get_path();
            file.file_open(&path.to_str().unwrap(), O_RDONLY).expect("failed to open");
            let fsize = file.file_size() as usize;
            file.file_close().expect("failed to close");
            if let Some(cache) = self.cache() {
                let page_cache_end = cache.end();
                cmp::max(page_cache_end, fsize)
            } else {
                fsize
            }
        } else {
            // DIR size should be 0
            0
        };
        log::debug!("file size: {}", size);
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode().bits() as _,
            st_nlink: inner.nlink() as u32,
            st_uid: 0,
            st_gid: 0,
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
        let mut file = self.file.lock();
        let ty = file.get_type();
        let size = if ty == InodeTypes::EXT4_DE_REG_FILE {
            let path = file.get_path();
            file.file_open(&path.to_str().unwrap(), O_RDONLY).expect("failed to open");
            let fsize = file.file_size() as usize;
            file.file_close().expect("failed to close");
            if let Some(cache) = self.cache() {
                let page_cache_end = cache.end();
                cmp::max(page_cache_end, fsize)
            } else {
                fsize
            }
        } else {
            // DIR size should be 0
            0
        };
        Xstat {
            stx_mask: mask.bits,
            stx_blksize: BLOCK_SIZE as _,
            stx_attributes: 0,
            stx_nlink: inner.nlink() as u32,
            stx_uid: 0,
            stx_gid: 0,
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

    fn symlink(&self, target_path: &str, link_path: &str) -> Result<Arc<dyn Inode>, SysError> {
        let file = self.file.lock();
        // create symlink
        file.symlink_create(target_path, link_path).map_err(|e| SysError::from_i32(e))?;
        // get the symlink Inode
        Ok(Arc::new(Ext4Inode::new(
            self.inode_inner().super_block.clone().unwrap(),
            link_path,
            InodeTypes::EXT4_DE_SYMLINK
        )))
    }

    fn link(&self, target_path: &str) -> Result<usize, SysError> {
        let file = self.file.lock();
        // create hard link
        file.link_create(target_path).expect("link create failed");
        Ok(0)
    }

    fn readlink(&self) -> Result<String, SysError> {
        let file = self.file.lock();
        let mut path_buf: Vec<u8> = vec![0u8; 512];
        let len = file.symlink_read(&mut path_buf).map_err(|e| SysError::from_i32(e))?;
        path_buf.truncate(len + 1);
        let path = CString::from_vec_with_nul(path_buf)
            .unwrap()
            .into_string()
            .unwrap();
        Ok(path)
    }

    /// remove the file that Ext4Inode holds
    fn unlink(&self) -> Result<usize, i32> {
        let mut file = self.file.lock();
        let itype = file.get_type();
        let cpath = file.get_path();
        let path = cpath.to_str().unwrap();
        match itype {
            InodeTypes::EXT4_DE_REG_FILE => {
                file.file_remove(path)
            }
            InodeTypes::EXT4_DE_DIR => {
                file.dir_rm(path)
            }
            _ => {
                panic!("not support");
            }
        }
    }

    fn remove(&self, name: &str, mode: InodeMode) -> Result<usize, i32> {
        let ty = InodeTypes::from(mode);
        let mut file = self.file.lock();
        let parent_path = String::from(file.get_path().to_str().unwrap());
        let fpath = rel_path_to_abs(&parent_path, name).unwrap();

        log::info!("[Ext4Inode] remove {}", fpath);
        //let fpath = self.path_deal_with(&fpath);
        let fpath = fpath.as_str();

        assert!(!fpath.is_empty()); // already check at `root.rs`

        match ty {
            InodeTypes::EXT4_DE_REG_FILE | InodeTypes::EXT4_DE_SYMLINK => {
                file.file_remove(fpath)
            }
            InodeTypes::EXT4_DE_DIR => {
                file.dir_rm(fpath)
            }
            _ => {
                panic!("not support");
            }
        }
    }

    fn rename(&self, target: &str, new_inode: Option<Arc<dyn Inode>>) -> Result<(), SysError> {
        let mut file = self.file.lock();
        let path = file.get_path();
        let old_path = path.to_str().expect("failed");
        let ty = file.get_type();
        let old_mode = InodeMode::from_inode_type(ty).get_type();
        log::debug!("old mode: {:x}", old_mode.bits());
        log::info!("[Ext4] rename {} -> {}", old_path, target);
        if let Some(new) = new_inode {
            let new_mode = new.inode_type();
            if new_mode != old_mode {
                return match (old_mode, new_mode) {
                    (InodeMode::FILE, InodeMode::DIR) => Err(SysError::EISDIR),
                    (InodeMode::DIR, InodeMode::FILE) => Err(SysError::ENOTDIR),
                    _ => unimplemented!(),
                };
            }
            let _ = match new_mode {
                InodeMode::FILE => file.file_remove(target),
                InodeMode::DIR => file.dir_rm(target),
                _ => unimplemented!(),
            };
        }
        let _ = match old_mode {
            InodeMode::FILE => file.file_rename(old_path, target),
            InodeMode::DIR => file.dir_mv(old_path, target),
            _ => unimplemented!(),
        };
        Ok(())
    }

    fn clean_cached(&self) {
        let cache = self.cache.clone();
        let mut pages = cache.get_pages().lock();
        for (_, page) in pages.iter_mut() {
            page.set_clean();
        }
    }
}

impl Drop for Ext4Inode {
    fn drop(&mut self) {
        // let mut file = self.file.lock();
        info!("Drop struct Inode");

        // flush the dirty page in page cache
        let cache = self.cache.clone();
        let mut pages = cache.get_pages().lock();
        for (&offset, page) in pages.iter_mut() {
            if page.is_dirty() == false {
                continue;
            }
            // info!("flush dirty page at offset {:#x}", offset);
            let buf_flush_size = cmp::min(cache.end() - offset, PAGE_SIZE);
            self.write_at(offset, &page.get_slice::<u8>()[..buf_flush_size]).expect("[PageCache]: failed at flush");
        }

        // file.file_close().expect("failed to close fd");
        // let _ = file; // todo
    }
}

/// translate between InodeTypes and InodeMode
impl InodeMode {
    pub fn from_inode_type(itype: InodeTypes) -> Self {
        let perm_mode = InodeMode::OWNER_MASK | InodeMode::GROUP_MASK | InodeMode::OTHER_MASK;
        let file_mode = match itype {
            InodeTypes::EXT4_DE_DIR => InodeMode::DIR,
            InodeTypes::EXT4_DE_REG_FILE => InodeMode::FILE,
            InodeTypes::EXT4_DE_CHRDEV => InodeMode::CHAR,
            InodeTypes::EXT4_DE_FIFO => InodeMode::FIFO,
            InodeTypes::EXT4_DE_BLKDEV => InodeMode::BLOCK,
            InodeTypes::EXT4_DE_SOCK => InodeMode::SOCKET,
            InodeTypes::EXT4_DE_SYMLINK => InodeMode::LINK,
            _ => InodeMode::TYPE_MASK,
        };
        file_mode | perm_mode
    }

    pub fn get_type(self) -> Self {
        self.intersection(InodeMode::TYPE_MASK)
    }
}

impl From<InodeMode> for InodeTypes {
    fn from(mode: InodeMode) -> Self {
        match mode.intersection(InodeMode::TYPE_MASK) {
            InodeMode::DIR => InodeTypes::EXT4_DE_DIR,
            InodeMode::FILE => InodeTypes::EXT4_DE_REG_FILE,
            InodeMode::LINK => InodeTypes::EXT4_DE_SYMLINK,
            InodeMode::CHAR => InodeTypes::EXT4_DE_CHRDEV,
            InodeMode::BLOCK => InodeTypes::EXT4_DE_BLKDEV,
            InodeMode::FIFO => InodeTypes::EXT4_DE_FIFO,
            InodeMode::SOCKET => InodeTypes::EXT4_DE_SOCK,
            _ => InodeTypes::EXT4_DE_UNKNOWN,
        }
    }
}

