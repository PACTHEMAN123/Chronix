//! Chronix universal file system interface
//! provide basic file operation

use alloc::sync::{Arc, Weak};

use crate::{fs::{tmpfs::{dentry::TmpDentry, inode::{InodeContent, TmpInode, TmpSysInode}}, vfs::{dentry::{global_find_dentry, global_update_dentry, global_update_path}, inode::InodeMode, Dentry, File, DCACHE}, OpenFlags, SuperBlock}, net::socket::{Socket, SocketDentry, SocketInode}, syscall::{SysError, SysResult}, utils::{abs_path_to_name, abs_path_to_parent}};

pub struct CNXFS;

/// simple interfaces
impl CNXFS {
    /// open a existed file
    pub fn open_file(absolute_path: &str) -> Result<Arc<dyn File>, SysError> {
        let dentry = global_find_dentry(absolute_path)?;
        let file = dentry.open(OpenFlags::empty()).ok_or(SysError::EINVAL)?;
        Ok(file)
    }

    /// try to open a file, create if not exist
    /// 1. the parent folder should exist
    /// 2. only support absolute path
    pub fn open_or_create_file(path: &str, mode: InodeMode) -> Result<Arc<dyn File>, SysError> {
        let dentry = global_find_dentry(path)?;
        if dentry.is_negative() {
            if abs_path_to_name(&path).unwrap() != abs_path_to_name(&dentry.path()).unwrap() {
                return Err(SysError::ENOENT);
            }
            let file = match mode.get_type() {
                InodeMode::FILE => {
                    let parent = dentry.parent().unwrap();
                    let new_inode = parent.inode()
                        .ok_or(SysError::EINVAL)?
                        .create(dentry.name(), mode)?;
                    dentry.set_inode(new_inode);
                    parent.add_child(dentry.clone());
                    dentry.open(OpenFlags::empty()).expect("should not failed")
                }
                InodeMode::SOCKET => {
                    let parent = dentry.parent().unwrap();
                    let new_inode= Arc::new(SocketInode::new());
                    let new_dentry = Arc::new(SocketDentry::new(dentry.name(), Some(parent.clone())));
                    new_dentry.set_inode(new_inode);
                    parent.add_child(new_dentry.clone());
                    DCACHE.lock().insert(new_dentry.path(), new_dentry.clone());
                    new_dentry.open(OpenFlags::empty()).expect("should not failed")
                }
                _ => {
                    todo!()
                }
            };
            Ok(file)
        } else {
            let file = dentry.open(OpenFlags::empty()).ok_or(SysError::EINVAL)?;
            return Ok(file)
        }
    }

    /// helper method to generate written dir
    pub fn create_sys_dir(name: &str, sb: Weak<dyn SuperBlock>, parent: Arc<dyn Dentry>) -> Arc<dyn Dentry> {
        let dentry = TmpDentry::new(name, Some(parent.clone()));
        let inode = TmpInode::new(sb.clone(), InodeMode::DIR);
        dentry.set_inode(inode);
        parent.add_child(dentry.clone());
        dentry
    }

    /// helper method to generate written file
    pub fn create_sys_file(contents: Arc<dyn InodeContent>, name: &str, parent: Arc<dyn Dentry>) -> Arc<dyn Dentry> {
        let dentry = TmpDentry::new(name, Some(parent.clone()));
        let inode = TmpSysInode::new(InodeMode::FILE, contents);
        dentry.set_inode(inode);
        parent.add_child(dentry.clone());
        dentry
    }
}