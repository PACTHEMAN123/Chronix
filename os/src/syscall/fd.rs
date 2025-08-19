use alloc::sync::Arc;

use crate::{fs::{tmpfs::{dentry::TmpDentry, inode::{EmptyFile, TmpSysInode}}, vfs::inode::InodeMode, OpenFlags}, syscall::{SysError, SysResult, SyscallId}, task::{current_task, fs::{FdFlags, FdInfo}}};

pub fn tmp_fd() -> Result<usize, SysError> {
    let task = current_task().unwrap().clone();
    let dentry = TmpDentry::new("fake", None);
    let inode = TmpSysInode::new(InodeMode::FILE, Arc::new(EmptyFile {}));
    dentry.set_inode(inode);
    let file = dentry.open(OpenFlags::empty()).unwrap();
    let fd = task.with_mut_fd_table(|t| t.alloc_fd())?;
    task.with_mut_fd_table(|t| t.put_file(fd, FdInfo { file, flags: FdFlags::empty() }))?;
    Ok(fd)
}

pub fn sys_allocfd(syscall_id: SyscallId) -> SysResult {
    log::warn!("fd syscall {:?} not implement", syscall_id);
    Ok(tmp_fd()? as isize)
}

