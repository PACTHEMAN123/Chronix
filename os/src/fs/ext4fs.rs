//! implement the vfs operations and node operations for ext4 filesystem
//! definition in `vfs.rs`

use core::cell::RefCell;
use core::ptr::NonNull;

use alloc::string::String;
use alloc::ffi::CString;
use super::disk::Disk;
use alloc::sync::Arc;
use alloc::vec::Vec;

use log::*;
use crate::logging;

use lwext4_rust::bindings::{
    O_APPEND, O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY, SEEK_CUR, SEEK_END, SEEK_SET,
};
use lwext4_rust::{Ext4BlockWrapper, Ext4File, InodeTypes, KernelDevOp};

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};

use crate::config::BLOCK_SIZE;

/// The Ext4 filesystem
#[allow(dead_code)]
pub struct Ext4FileSystem {
    inner: Ext4BlockWrapper<Disk>,
    root: Arc<Inode>,
}

unsafe impl Sync for Ext4FileSystem {}
unsafe impl Send for Ext4FileSystem {}

impl Ext4FileSystem {
    /// Create a new Ext4 filesystem
    pub fn new(disk: Disk) -> Self {
        info!(
            "Got Disk size:{}, position:{}",
            disk.size(),
            disk.position()
        );
        let inner = Ext4BlockWrapper::<Disk>::new(disk)
            .expect("failed to initialize EXT4 filesystem");
        let root = Arc::new(Inode::new("/", InodeTypes::EXT4_DE_DIR));
        Self { inner, root }
    }

    /// Get the root directory
    pub fn root_dir(&self) -> Arc<Inode> {
        info!("trying to get the root dir");
        Arc::clone(&self.root)
    }
}

/// The inode of the Ext4 filesystem
pub struct Inode(RefCell<Ext4File>);

unsafe impl Send for Inode {}
unsafe impl Sync for Inode {}

impl Inode {
    /// Create a new inode
    pub fn new(path: &str, types: InodeTypes) -> Self {
        info!("Inode new {:?} {}", types, path);
        //file.file_read_test("/test/test.txt", &mut buf);

        Self(RefCell::new(Ext4File::new(path, types)))
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
        let file = self.0.borrow_mut();
        let path = file.get_path();
        let fpath = String::from(path.to_str().unwrap().trim_end_matches('/')) + "/" + p;
        info!("dealt with full path: {}", fpath.as_str());
        fpath
    }

    /// Find inode under current inode by name
    #[allow(unused)]
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let file = self.0.borrow_mut();
        info!("find name: {} in {}", name, file.get_path().to_str().unwrap());
        let (names, inode_type) = file.lwext4_dir_entries().unwrap();
        info!("out lwext4_dir_entries");
        let mut name_iter = names.iter();
        let mut inode_type_iter = inode_type.iter();

        info!("into while");
        while let Some(iname) = name_iter.next() {
            let itypes = inode_type_iter.next();
            info!("iname: {}", core::str::from_utf8(iname).unwrap());
            if core::str::from_utf8(iname).unwrap().trim_end_matches('\0') == name {
                info!("find {} success", name);

                // lwext4 needs full path
                let full_path = String::from(file.get_path().to_str().unwrap().trim_end_matches('/')) + "/" + name;
                return Some(Arc::new(Inode::new(full_path.as_str(), itypes.unwrap().clone())));
            }
        }

        info!("find {} failed", name);
        None
    }

    /// Look up the node with given `name` in the directory
    /// Return the node if found.
    pub fn lookup(&self, name: &str) -> Option<Arc<Inode>> {
        let mut file = self.0.borrow_mut();
        
        let full_path = String::from(file.get_path().to_str().unwrap().trim_end_matches('/')) + "/" + name;
        
        if file.check_inode_exist(full_path.as_str(), InodeTypes::EXT4_DE_REG_FILE) {
            info!("lookup {} success", name);
            return Some(Arc::new(Inode::new(full_path.as_str(), InodeTypes::EXT4_DE_REG_FILE)));
        }

        // todo!: add support for directory

        info!("lookup {} failed", name);
        None
    }

    /// list all files' name in the directory
    pub fn ls(&self) -> Vec<String> {
        let file = self.0.borrow_mut();

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
            names.push(String::from(core::str::from_utf8(iname).unwrap()));
        }
        names
    }

    /// Read data from inode at offset
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32> {
        debug!("To read_at {}, buf len={}", offset, buf.len());
        let mut file = self.0.borrow_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY)?;

        file.file_seek(offset as i64, SEEK_SET)?;
        let r = file.file_read(buf);

        let _ = file.file_close();
        r
    }

    /// Write data to inode at offset
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize, i32> {
        debug!("To write_at {}, buf len={}", offset, buf.len());
        let mut file = self.0.borrow_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR)?;

        file.file_seek(offset as i64, SEEK_SET)?;
        let r = file.file_write(buf);

        let _ = file.file_close();
        r
    }

    /// Truncate the inode to the given size
    pub fn truncate(&self, size: u64) -> Result<usize, i32> {
        info!("truncate file to size={}", size);
        let mut file = self.0.borrow_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR)?;

        let t = file.file_truncate(size);

        let _ = file.file_close();
        t
    }

    /// Create a new inode and return the inode
    pub fn create(&self, path: &str, ty: InodeTypes) -> Option<Arc<Inode>> {
        info!("create {:?} on Ext4fs: {}", ty, path);
        let fpath = self.path_deal_with(path);
        let fpath = fpath.as_str();
        if fpath.is_empty() {
            info!("given path is empty");
            return None;
        }

        let types = ty;

        let mut file = self.0.borrow_mut();

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
                Some(Arc::new(Inode::new(fpath, types)))
            }
        }
    }

    /// Remove the inode
    #[allow(unused)]
    fn remove(&self, path: &str) -> Result<usize, i32> {
        info!("remove ext4fs: {}", path);
        let fpath = self.path_deal_with(path);
        let fpath = fpath.as_str();

        assert!(!fpath.is_empty()); // already check at `root.rs`

        let mut file = self.0.borrow_mut();
        if file.check_inode_exist(fpath, InodeTypes::EXT4_DE_DIR) {
            // Recursive directory remove
            file.dir_rm(fpath)
        } else {
            file.file_remove(fpath)
        }
    }

    /// Get the parent directory of this directory.
    /// Return `None` if the node is a file.
    #[allow(unused)]
    fn parent(&self) -> Option<Arc<Inode>> {
        let file = self.0.borrow_mut();
        if file.get_type() == InodeTypes::EXT4_DE_DIR {
            let path = file.get_path();
            let path = path.to_str().unwrap();
            info!("Get the parent dir of {}", path);
            let path = path.trim_end_matches('/').trim_end_matches(|c| c != '/');
            if !path.is_empty() {
                return Some(Arc::new(Self::new(path, InodeTypes::EXT4_DE_DIR)));
            }
        }
        None
    }

    /// Rename the inode
    #[allow(unused)]
    fn rename(&self, src_path: &str, dst_path: &str) -> Result<usize, i32> {
        info!("rename from {} to {}", src_path, dst_path);
        let mut file = self.0.borrow_mut();
        file.file_rename(src_path, dst_path)
    }
}

impl Drop for Inode {
    fn drop(&mut self) {
        let mut file = self.0.borrow_mut();
        info!("Drop struct Inode {:?}", file.get_path());
        file.file_close().expect("failed to close fd");
        drop(file); // todo
    }
}