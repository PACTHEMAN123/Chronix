//! Chronix universal file system interface
//! provide basic file operation

use alloc::sync::{Arc, Weak};

use crate::{fs::{tmpfs::{dentry::TmpDentry, inode::{InodeContent, TmpInode, TmpSysInode}}, vfs::{dentry::global_find_dentry, inode::InodeMode, Dentry, File}, OpenFlags, SuperBlock}, syscall::{SysError, SysResult}, utils::{abs_path_to_name, abs_path_to_parent}};

pub struct CNXFS;

/// simple interfaces
impl CNXFS {
    /// open a existed file
    pub fn open_file(absolute_path: &str) -> Result<Arc<dyn File>, SysError> {
        let dentry = global_find_dentry(absolute_path)?;
        let file = dentry.open(OpenFlags::empty()).ok_or(SysError::EINVAL)?;
        Ok(file)
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