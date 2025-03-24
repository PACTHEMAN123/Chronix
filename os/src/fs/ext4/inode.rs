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
use alloc::vec::Vec;

use log::*;
use crate::fs::page::cache::PageCache;
use crate::fs::page::page::{Page, PAGE_SIZE};
use crate::fs::vfs::inode::InodeMode;
use crate::fs::vfs::{InodeInner, Inode};
use crate::fs::{Kstat, SuperBlock};
use crate::sync::UPSafeCell;

use lwext4_rust::bindings::{
    O_APPEND, O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY, SEEK_CUR, SEEK_END, SEEK_SET,
};
use lwext4_rust::{Ext4BlockWrapper, Ext4File, InodeTypes, KernelDevOp};

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};

use crate::config::BLOCK_SIZE;

/// The inode of the Ext4 filesystem
pub struct Ext4Inode {
    inner: InodeInner,
    file: UPSafeCell<Ext4File>,
    cache: Arc<PageCache>,
}

unsafe impl Send for Ext4Inode {}
unsafe impl Sync for Ext4Inode {}

impl Ext4Inode {
    /// Create a new inode
    pub fn new(super_block: Arc<dyn SuperBlock>, path: &str, types: InodeTypes) -> Self {
        //info!("Inode new {:?} {}", types, path);
        let mode = InodeMode::from_inode_type(types.clone());
        let mut file  = Ext4File::new(path, types);
        // (todo) notice that lwext4 mention in file_size(): should open file as RDONLY first 
        // may be a bug in the future
        let size = file.file_size();
        Self {
            inner: InodeInner::new(super_block.clone(), mode, size as usize),
            file: UPSafeCell::new(file),
            cache: Arc::new(PageCache::new()),
        }
    }

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
        let file = self.file.exclusive_access();
        let path = file.get_path();
        let fpath = String::from(path.to_str().unwrap().trim_end_matches('/')) + "/" + p;
        info!("dealt with full path: {}", fpath.as_str());
        fpath
    }
}

impl Inode for Ext4Inode {

    fn inner(&self) -> &InodeInner {
        &self.inner
    }

    /// Look up the node with given `name` in the directory
    /// Return the node if found.
    fn lookup(&self, name: &str) -> Option<Arc<dyn Inode>> {
        let file = self.file.exclusive_access();
        
        let full_path = String::from(file.get_path().to_str().unwrap().trim_end_matches('/')) + "/" + name;
        
        if file.check_inode_exist(full_path.as_str(), InodeTypes::EXT4_DE_REG_FILE) {
            //info!("lookup {} success", name);
            return Some(Arc::new(Ext4Inode::new(
                self.inner().super_block.upgrade()?.clone(), 
                full_path.as_str(), 
                InodeTypes::EXT4_DE_REG_FILE)));
        } else if file.check_inode_exist(full_path.as_str(), InodeTypes::EXT4_DE_DIR) {
            info!("lookup dir {} success", name);
            return Some(Arc::new(Ext4Inode::new(
                self.inner().super_block.upgrade()?.clone(), 
                full_path.as_str(), 
                InodeTypes::EXT4_DE_DIR)));
        }

        // todo!: add support for directory

        //info!("lookup {} failed", name);
        None
    }

    /// list all files' name in the directory
    fn ls(&self) -> Vec<String> {
        let file = self.file.exclusive_access();

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
        let file = self.file.exclusive_access();
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
        let file =  self.file.exclusive_access();
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

        while buf_offset < buf.len() {
            let cache = self.cache.clone();
            //info!("current offset: {}, file end: {}", current_offset, cache.end());
            if current_offset > cache.end() {
                break;
            }
            let page_offset = current_offset / PAGE_SIZE * PAGE_SIZE;
            let in_page_offset = current_offset % PAGE_SIZE;

            // get the cached page or read page using IO and store in cache
            
            let page = if let Some(page) = cache.get_page(page_offset) {
                //info!("[PAGE CACHE]: hit at offset: {:x}", page_offset);
                page.clone()
            } else {
                //info!("[PAGE CACHE]: miss at offset: {:x}", page_offset);
                // direct read at the offset of page size
                let mut page = Page::new(page_offset);
                let read_size = Arc::get_mut(&mut page).unwrap().read_from(self.clone(), offset);
                cache.insert_page(page_offset, page.clone());
                cache.update_end(page_offset + read_size);
                page
            };

            // now use the page to fill in the buf
            let page_read_size = page.read_at(in_page_offset, &mut buf[buf_offset..]);
            //info!("read at offset: {}, read_size: {}", in_page_offset, page_read_size);

            total_read_size += page_read_size;
            buf_offset += page_read_size;
            current_offset += page_read_size; 
        }

        Ok(total_read_size)
    }

    fn cache_write_at(self: Arc<Self>, offset: usize, buf: &[u8]) -> Result<usize, i32> {
        let file = self.file.exclusive_access();
        let cpath = file.get_path();
        let path = cpath.to_str().unwrap();
        file.file_open(path, O_RDWR)?;
        
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
                //info!("[PAGE CACHE]: hit at offset: {}", page_offset);
                page.clone()
            } else {
                //info!("[PAGE CACHE]: miss at offset: {}", page_offset);
                // unlike read, no need to read out the data
                // just simply cache data and write back when inode drop
                let page = Page::new(page_offset);
                cache.insert_page(page_offset, page.clone());
                page
            };

            // now use the buf to fill in the page
            let page_write_size = page.write_at(in_page_offset, &buf[buf_offset..]);
            page.set_dirty();
            cache.update_end(page_offset + page_write_size);

            total_write_size += page_write_size;
            buf_offset += page_write_size;
            current_offset += page_write_size;
        }

        Ok(total_write_size)
    }

    /// Truncate the inode to the given size
    fn truncate(&self, size: u64) -> Result<usize, i32> {
        info!("truncate file to size={}", size);
        let file = self.file.exclusive_access();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR)?;

        let t = file.file_truncate(size);

        let _ = file.file_close();
        t
    }

    /// Create a new inode and return the inode
    fn create(&self, name: &str, mode: InodeMode) -> Option<Arc<dyn Inode>> {
        let ty: InodeTypes = mode.into();
        let file = self.file.exclusive_access();
        let parent_path = file.get_path().to_str().expect("cpath failed").to_string();
        let fpath = parent_path + "/" + name;
        info!("create {:?} on Ext4fs: {}", ty, fpath);
        let fpath = self.path_deal_with(&fpath);
        let fpath = fpath.as_str();
        if fpath.is_empty() {
            info!("given path is empty");
            return None;
        }

        let types = ty;

        let file = self.file.exclusive_access();

        let result = if file.check_inode_exist(fpath, types.clone()) {
            info!("inode already exists");
            Ok(0)
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
                None
            }
            Ok(_) => {
                info!("create inode success");
                Some(Arc::new(Ext4Inode::new(
                    self.inner().super_block.upgrade()?.clone(),
                    fpath, types)))
            }
        }
    }

    fn getattr(&self) -> Kstat {
        let inner = self.inner();
        let size = inner.size;
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode.bits() as _,
            st_nlink: inner.nlink as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: size as _,
            _pad1: 0,
            st_blksize: BLOCK_SIZE as _,
            st_blocks: (size / BLOCK_SIZE) as _,
            st_atime_sec: inner.atime.tv_sec as _,
            st_atime_nsec: inner.atime.tv_nsec as _,
            st_mtime_sec: inner.mtime.tv_sec as _,
            st_mtime_nsec: inner.mtime.tv_nsec as _,
            st_ctime_sec: inner.ctime.tv_sec as _,
            st_ctime_nsec: inner.ctime.tv_nsec as _,

        }
    }

    /// remove the file that Ext4Inode holds
    fn unlink(&self) -> Result<usize, i32> {
        let file = self.file.exclusive_access();
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
        let file = self.file.exclusive_access();
        let parent_path = String::from(file.get_path().to_str().unwrap());
        let fpath = parent_path + "/" + name;

        info!("remove ext4fs: {}", fpath);
        let fpath = self.path_deal_with(&fpath);
        let fpath = fpath.as_str();

        assert!(!fpath.is_empty()); // already check at `root.rs`

        if file.check_inode_exist(fpath, ty) {
            // Recursive directory remove
            file.dir_rm(fpath)
        } else {
            file.file_remove(fpath)
        }
    }

}

impl Drop for Ext4Inode {
    fn drop(&mut self) {
        let file = self.file.exclusive_access();
        info!("Drop struct Inode {:?}", file.get_path());

        // flush the dirty page in page cache
        let cache = self.cache.clone();
        let mut pages = cache.get_pages().lock();
        for (&offset, page) in pages.iter_mut() {
            if page.is_dirty() == false {
                continue;
            }
            self.write_at(offset, page.get_slice::<u8>()).expect("[PageCache]: failed at flush");
        }

        file.file_close().expect("failed to close fd");
        let _ = file; // todo
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

