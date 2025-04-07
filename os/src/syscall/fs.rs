//! File and filesystem-related syscalls
use alloc::{string::ToString, sync::Arc};
use hal::{addr::VirtAddr, instruction::{Instruction, InstructionHal}};
use log::{info, warn};
use strum::FromRepr;
use virtio_drivers::PAGE_SIZE;
use crate::{drivers::BLOCK_DEVICE, fs::{
    get_filesystem, pipe::make_pipe, vfs::{dentry::{self, global_find_dentry}, file::{open_file, SeekFrom}, fstype::MountFlags, inode::InodeMode, Dentry, DentryState, File}, Kstat, OpenFlags, UtsName, Xstat, XstatMask, AT_FDCWD, AT_REMOVEDIR
}, processor::context::SumGuard, task::{fs::{FdFlags, FdInfo}, task::TaskControlBlock}};
use crate::utils::{
    path::*,
    string::*,
};
use super::{SysResult,SysError};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::processor::processor::{current_processor,current_task,current_user_token};

/// syscall: write
pub async fn sys_write(fd: usize, buf: usize, len: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    //info!("task {} trying to write fd {}", task.gettid(), fd);
    let file = task.with_fd_table(|table| table.get_file(fd))?;
    let _sum_guard = SumGuard::new();
    let buf = unsafe {
        core::slice::from_raw_parts::<u8>(buf as *const u8, len)
    };
    return Ok(file.write(buf).await as isize);
}


/// syscall: read
pub async fn sys_read(fd: usize, buf: usize, len: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    //info!("task {} trying to read fd {}", task.gettid(), fd);
    let file = task.with_fd_table(|table| table.get_file(fd))?;
    let _sum_guard = SumGuard::new();
    let buf = unsafe {
        core::slice::from_raw_parts_mut::<u8>(buf as *mut u8, len)
    };
    let ret = file.read(buf).await;
    return Ok(ret as isize);
}

/// syscall: close
pub fn sys_close(fd: usize) -> SysResult {
    let task = current_task().unwrap();
    task.with_mut_fd_table(|table| table.remove(fd))?;
    Ok(0)
}

/// syscall: getcwd
/// The getcwd() function copies an absolute pathname of 
/// the current working directory to the array pointed to by buf, 
/// which is of length size.
/// On success, these functions return a pointer to 
/// a string containing the pathname of the current working directory. 
/// In the case getcwd() and getwd() this is the same value as buf.
/// On failure, these functions return NULL, 
/// and errno is set to indicate the error. 
/// The contents of the array pointed to by buf are undefined on error.
pub fn sys_getcwd(buf: usize, len: usize) -> SysResult {
    let _sum_guard = SumGuard::new();
    let task = current_task().unwrap();
    task.with_cwd(|cwd| {
        let path = cwd.path();
        if len < path.len() + 1 {
            info!("[sys_getcwd]: buf len too small to recv path");
            return Err(SysError::ERANGE);
        } else {
            //info!("copying path: {}, len: {}", path, path.len());
            let new_buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
            new_buf.fill(0 as u8);
            let new_buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, path.len()) };
            new_buf.copy_from_slice(path.as_bytes());
            return Ok(buf as isize);
        }
    })
}

/// syscall: dup
pub fn sys_dup(old_fd: usize) -> SysResult {
    let task = current_task().unwrap();
    let new_fd = task.with_mut_fd_table(|table| table.dup_no_flag(old_fd))?;
    Ok(new_fd as isize)
}

/// syscall: dup3
pub fn sys_dup3(old_fd: usize, new_fd: usize, flags: u32) -> SysResult {
    //info!("dup3: old_fd = {}, new_fd = {}", old_fd, new_fd);
    let task = current_task().unwrap();
    let flags = OpenFlags::from_bits(flags as i32).ok_or(SysError::EINVAL)?;
    if old_fd == new_fd {
        return Err(SysError::EINVAL);
    }
    let new_fd = task.with_mut_fd_table(|table| table.dup3(old_fd, new_fd, flags.into()))?;
    Ok(new_fd as isize)
}

/// syscall: openat
/// If the pathname given in pathname is relative, 
/// then it is interpreted relative to the directory referred to by the file descriptor dirfd 
/// (rather than relative to the current working directory of the calling process, 
/// as is done by open(2) for a relative pathname).
/// If pathname is relative and dirfd is the special value AT_FDCWD, 
/// then pathname is interpreted relative to the current working directory of the calling process (like open(2)).
/// If pathname is absolute, then dirfd is ignored.
pub fn sys_openat(dirfd: isize, pathname: *const u8, flags: u32, _mode: u32) -> SysResult {
    let flags = OpenFlags::from_bits(flags as i32).unwrap();
    let task = current_task().unwrap().clone();

    if let Some(path) = user_path_to_string(pathname) {
        let dentry = at_helper(task.clone(), dirfd, pathname, flags)?;
        info!("trying to open {}", dentry.path());
        if flags.contains(OpenFlags::O_CREAT) {
            // inode not exist, create it as a regular file
            if flags.contains(OpenFlags::O_EXCL) && dentry.state() != DentryState::NEGATIVE {
                return Err(SysError::EEXIST);
            }
            let parent = dentry.parent().expect("[sys_openat]: can not open root as file!");
            let name = abs_path_to_name(&path).unwrap();
            info!("name: {}", name);
            let new_inode = parent.inode().unwrap().create(&name, InodeMode::FILE).unwrap();
            dentry.set_inode(new_inode);
            dentry.set_state(DentryState::USED);
        }
        if dentry.state() == DentryState::NEGATIVE {
            return Err(SysError::ENOENT);
        }
        let inode = dentry.inode().unwrap();
        if flags.contains(OpenFlags::O_DIRECTORY) && inode.inner().mode.get_type() != InodeMode::DIR {
            return Err(SysError::ENOTDIR);
        }
        let file = dentry.open(flags).unwrap();
        file.set_flags(flags);
        let fd = task.with_mut_fd_table(|table| table.alloc_fd());
        let fd_info = FdInfo { file, flags: flags.into() };
        task.with_mut_fd_table(|t|t.put_file(fd, fd_info))?;
        return Ok(fd as isize)
    } else {
        info!("[sys_openat]: pathname is empty!");
        return Err(SysError::ENOENT);
    }
}

/// syscall: mkdirat
/// If the pathname given in pathname is relative, 
/// then it is interpreted relative to the directory referred to by the file descriptor dirfd 
/// (rather than relative to the current working directory of the calling process, 
/// as is done by mkdir(2) for a relative pathname).
/// If pathname is relative and dirfd is the special value AT_FDCWD, 
/// then pathname is interpreted relative to the current working directory of the calling process (like mkdir(2)).
/// If pathname is absolute, then dirfd is ignored.
pub fn sys_mkdirat(dirfd: isize, pathname: *const u8, _mode: usize) -> SysResult {
    if let Some(path) = user_path_to_string(pathname) {
        let task = current_task().unwrap().clone();
        let dentry = at_helper(task, dirfd, pathname, OpenFlags::empty())?;
        if dentry.state() != DentryState::NEGATIVE {
            return Err(SysError::EEXIST);
        }
        let parent = dentry.parent().unwrap();
        let name = abs_path_to_name(&path).unwrap();
        let new_inode = parent.inode().unwrap().create(&name, InodeMode::DIR).unwrap();
        dentry.set_inode(new_inode);
        dentry.set_state(DentryState::USED);
    } else {
        warn!("[sys_mkdirat]: pathname is empty!");
        return Err(SysError::ENOENT);
    }
    Ok(0)
}

const AT_SYMLINK_NOFOLLOW: i32 = 0x100;

/// syscall: fstatat
pub fn sys_fstatat(dirfd: isize, pathname: *const u8, stat_buf: usize, flags: i32) -> SysResult {
    let _sum_guard= SumGuard::new();
    
    let task = current_task().unwrap().clone();
    let dentry = if flags == AT_SYMLINK_NOFOLLOW {
        at_helper(task.clone(), dirfd, pathname, OpenFlags::O_NOFOLLOW)?
    } else {
        at_helper(task.clone(), dirfd, pathname, OpenFlags::empty())?
    };
    if dentry.state() == DentryState::NEGATIVE {
        return Err(SysError::EBADF);
    }

    let stat = dentry.inode().unwrap().getattr();
    let stat_ptr = stat_buf as *mut Kstat;
    unsafe {
        Instruction::set_sum();
        stat_ptr.write(stat);
    }
    Ok(0)
}

/// chdir() changes the current working directory of the calling
/// process to the directory specified in path.
/// On success, zero is returned.  On error, -1 is returned, and errno
/// is set to indicate the error.
pub fn sys_chdir(path: *const u8) -> SysResult {
    let path = user_path_to_string(path).unwrap();
    let dentry = global_find_dentry(&path);
    if dentry.state() == DentryState::NEGATIVE {
        info!("[sys_chdir]: dentry not found");
        return Err(SysError::ENOENT);
    } else {
        let task = current_task().unwrap().clone();
        task.set_cwd(dentry);
        return Ok(0);
    }
}


const PIPE_BUF_LEN: usize = PAGE_SIZE;
/// pipe() creates a pipe, a unidirectional data channel 
/// that can be used for interprocess communication. 
/// The array pipefd is used to return two file descriptors 
/// referring to the ends of the pipe. 
/// pipefd[0] refers to the read end of the pipe. 
/// pipefd[1] refers to the write end of the pipe. 
/// Data written to the write end of the pipe is buffered by the kernel 
/// until it is read from the read end of the pipe.
/// todo: support flags
pub fn sys_pipe2(pipe: *mut i32, flags: u32) -> SysResult {
    let task = current_task().unwrap().clone();
    let flags = OpenFlags::from_bits(flags as i32).unwrap();
    let (read_file, write_file) = make_pipe(PIPE_BUF_LEN);
    let read_fd = task.with_mut_fd_table(|t|t.alloc_fd());
    task.with_mut_fd_table(|t| t.put_file(read_fd, FdInfo { file: read_file, flags: flags.into() }))?;
    let write_fd = task.with_mut_fd_table(|t|t.alloc_fd());
    task.with_mut_fd_table(|t| t.put_file(write_fd, FdInfo { file: write_file, flags: flags.into() }))?;

    let _sum = SumGuard::new();
    let pipefd = unsafe { core::slice::from_raw_parts_mut(pipe, 2 * core::mem::size_of::<i32>()) };
    info!("read fd: {}, write fd: {}", read_fd, write_fd);
    pipefd[0] = read_fd as i32;
    pipefd[1] = write_fd as i32;
    Ok(0)
}

/// syscall fstat
pub fn sys_fstat(fd: usize, stat_buf: usize) -> SysResult {
    let _sum_guard = SumGuard::new();
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let stat = file.dentry().unwrap().inode().unwrap().getattr();
    let stat_ptr = stat_buf as *mut Kstat;
    unsafe {
        Instruction::set_sum();
        *stat_ptr = stat;
    }
    return Ok(0);
}

/// syscall statx
pub fn sys_statx(dirfd: isize, pathname: *const u8, flags: i32, mask: u32, statx_buf: VirtAddr) -> SysResult {
    let _sum_guard = SumGuard::new();
    let flags = OpenFlags::from_bits(flags).ok_or(SysError::EINVAL)?;
    let mask = XstatMask::from_bits_truncate(mask);
    let task = current_task().unwrap().clone();
    let dentry = at_helper(task, dirfd, pathname, flags)?;
    let inode = dentry.inode().unwrap();
    let statx_ptr = statx_buf.0 as *mut Xstat;
    let statx = inode.getxattr(mask);
    unsafe {
        statx_ptr.write(statx);
    }
    Ok(0)
}

/// syscall uname
pub fn sys_uname(uname_buf: usize) -> SysResult {
    let _sum_guard = SumGuard::new();
    let uname = UtsName::default();
    let uname_ptr = uname_buf as *mut UtsName;
    unsafe {
        *uname_ptr = uname;
    }
    Ok(0)
}



#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct LinuxDirent64 {
    d_ino: u64,
    d_off: u64,
    d_reclen: u16,
    d_type: u8,
    // d_name follows here, which will be written later
}
/// syscall getdents
/// ssize_t getdents64(int fd, void dirp[.count], size_t count);
/// The system call getdents() reads several linux_dirent structures
/// from the directory referred to by the open file descriptor fd into
/// the buffer pointed to by dirp.  The argument count specifies the
/// size of that buffer.
/// (todo) now mostly copy from Phoenix
pub fn sys_getdents64(fd: usize, buf: usize, len: usize) -> SysResult {
    const LEN_BEFORE_NAME: usize = 19;
    let task = current_task().unwrap().clone();
    let _sum_guard = SumGuard::new();
    let buf_slice = unsafe {
        core::slice::from_raw_parts_mut(buf as *mut u8, len)
    };
    assert!(buf_slice.len() == len);

    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let dentry = file.dentry().unwrap();
    let mut buf_it = buf_slice;
    let mut writen_len = 0;
    for child in dentry.child_dentry().iter().skip(file.pos()) {
        assert!(child.state() != DentryState::NEGATIVE);
        // align to 8 bytes
        let c_name_len = child.name().len() + 1;
        let rec_len = (LEN_BEFORE_NAME + c_name_len + 7) & !0x7;
        let inode = child.inode().unwrap();
        let linux_dirent = LinuxDirent64 {
            d_ino: inode.inner().ino as u64,
            d_off: file.pos() as u64,
            d_type: inode.inner().mode.bits() as u8,
            d_reclen: rec_len as u16,
        };

        //info!("[sys_getdents64] linux dirent {linux_dirent:?}");
        if writen_len + rec_len > len {
            break;
        }

        file.seek(SeekFrom::Current(1))?;
        let ptr = buf_it.as_mut_ptr() as *mut LinuxDirent64;
        unsafe {
            ptr.copy_from_nonoverlapping(&linux_dirent, 1);
        }
        buf_it[LEN_BEFORE_NAME..LEN_BEFORE_NAME + c_name_len - 1]
            .copy_from_slice(child.name().as_bytes());
        buf_it[LEN_BEFORE_NAME + c_name_len - 1] = b'\0';
        buf_it = &mut buf_it[rec_len..];
        writen_len += rec_len;
    }
    info!("writen_len: {}", writen_len);
    return Ok(writen_len as isize);
}

/// unlink() deletes a name from the filesystem.  If that name was the
/// last link to a file and no processes have the file open, the file
/// is deleted and the space it was using is made available for reuse.
/// If the name was the last link to a file but any processes still
/// have the file open, the file will remain in existence until the
/// last file descriptor referring to it is closed.
/// If the name referred to a symbolic link, the link is removed.
/// If the name referred to a socket, FIFO, or device, the name for it
/// is removed but processes which have the object open may continue to use it.
/// (todo): now only remove, but not check for remaining referred.
pub fn sys_unlinkat(dirfd: isize, pathname: *const u8, flags: i32) -> SysResult {
    let task = current_task().unwrap().clone();
    let path = user_path_to_string(pathname).unwrap();
    let dentry = at_helper(task, dirfd, pathname, OpenFlags::O_NOFOLLOW)?;
    if dentry.parent().is_none() {
        warn!("cannot unlink root!");
        return Err(SysError::ENOENT);
    }
    let inode = dentry.inode().unwrap();
    let is_dir = inode.inner().mode == InodeMode::DIR;
    if flags == AT_REMOVEDIR && !is_dir {
        return Err(SysError::ENOTDIR);
    } else if flags != AT_REMOVEDIR && is_dir {
        return Err(SysError::EPERM);
    }
    // use parent inode to remove the inode in the fs
    let name = abs_path_to_name(&path).unwrap();
    dentry.parent().unwrap().inode().unwrap().remove(&name, inode.inner().mode).expect("remove failed");
    //inode.unlink().expect("inode unlink failed");
    dentry.clear_inode();
    Ok(0)
}

/// syscall: mount
/// (todo)
pub fn sys_mount(
    _source: *const u8,
    _target: *const u8,
    _fstype: *const u8,
    _flags: u32,
    _data: usize,
) -> SysResult {
    /*
    let _source_path = user_path_to_string(source).unwrap();
    let target_path = user_path_to_string(target).unwrap();
    let flags = MountFlags::from_bits(flags).unwrap();
    let fat32_type = get_filesystem("fat32");
    let dev = Some(BLOCK_DEVICE.clone());
    let parent_path = abs_path_to_parent(&target_path).unwrap();
    let name = abs_path_to_name(&target_path).unwrap();
    let parent = global_find_dentry(&parent_path);

    fat32_type.mount(&name, Some(parent), flags, dev);
    */
    Ok(0)
}

/// fake unmount
pub fn sys_umount2(_target: *const u8, _flags: u32) -> SysResult {
    Ok(0)
}

/// syscall: ioctl
pub fn sys_ioctl(fd: usize, cmd: usize, arg: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    unsafe {
        Instruction::set_sum();
    }
    file.ioctl(cmd, arg)
}

#[derive(FromRepr, Debug, Eq, PartialEq, Clone, Copy, Default)]
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[repr(isize)]
pub enum FcntlOp {
    F_DUPFD = 0,
    F_DUPFD_CLOEXEC = 1030,
    F_GETFD = 1,
    F_SETFD = 2,
    F_GETFL = 3,
    F_SETFL = 4,
    #[default]
    F_UNIMPL,
}

/// syscall: fcntl
pub fn sys_fnctl(fd: usize, op: isize, arg: usize) -> SysResult {
    let op = FcntlOp::from_repr(op).unwrap_or_default();
    let task = current_task().unwrap().clone();
    match op {
        FcntlOp::F_DUPFD => {
            // Duplicate the file descriptor fd using the lowest-numbered
            // available file descriptor greater than or equal to arg.
            let new_fd = task.with_mut_fd_table(|t| t.dup_with_bound(fd, arg, OpenFlags::empty().into()))?;
            Ok(new_fd as isize)
        }
        FcntlOp::F_DUPFD_CLOEXEC => {
            // As for F_DUPFD, but additionally set the close-on-exec flag
            // for the duplicate file descriptor.
            let new_fd = task.with_mut_fd_table(|t| t.dup_with_bound(fd, arg, FdFlags::CLOEXEC))?;
            Ok(new_fd as isize)
        }
        FcntlOp::F_GETFD => {
            // Return (as the function result) the file descriptor flags;
            // arg is ignored.
            let fd_info = task.with_fd_table(|t| t.get_fd_info(fd))?;
            Ok(fd_info.flags().bits() as isize)
        }
        FcntlOp::F_SETFD => {
            let arg = OpenFlags::from_bits_truncate(arg as i32);
            let fd_flags = FdFlags::from(arg);
            task.with_mut_fd_table(|table| {
                let fd_info = table.get_mut_fd_info(fd)?;
                fd_info.set_flags(fd_flags);
                Ok(0)
            })
        }
        FcntlOp::F_GETFL => {
            let file = task.with_fd_table(|table| table.get_file(fd))?;
            Ok(file.flags().bits() as _)
        }
        FcntlOp::F_SETFL => {
            let flags = OpenFlags::from_bits_truncate(arg as _);
            let file = task.with_fd_table(|table| table.get_file(fd))?;
            file.set_flags(flags.status());
            Ok(0)
        }
        _ => {
            log::warn!("fcntl cmd: {op:?} not implemented");
            Ok(0)
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(missing_docs)]
pub struct IoVec {
    pub base: usize,
    pub len: usize,
}

/// The readv() system call reads iovcnt buffers from the file
/// associated with the file descriptor fd into the buffers described
/// by iov ("scatter input").
pub async fn sys_readv(fd: usize, iov: usize, iovcnt: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let iovs = unsafe {
        Instruction::set_sum();
        core::slice::from_raw_parts(iov as *const IoVec, iovcnt)
    };
    let mut totol_len = 0usize;
    for (i, iov) in iovs.iter().enumerate() {
        if iov.len == 0 {
            continue;
        }
        log::debug!("[sys_writev]: iov[{}], ptr: {:#x}, len: {}", i, iov.base, iov.len);
        let buf = unsafe {
            Instruction::set_sum();
            core::slice::from_raw_parts_mut(iov.base as *mut u8, iov.len)
        };
        let read_len = file.read(buf).await;
        totol_len += read_len;
    }
    Ok(totol_len as isize)
}

/// The writev() function shall be equivalent to write(), except as
/// described below. The writev() function shall gather output data
/// from the iovcnt buffers specified by the members of the iov array:
/// iov[0], iov[1], ..., iov[iovcnt-1].
pub async fn sys_writev(fd: usize, iov: usize, iovcnt: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let iovs = unsafe {
        Instruction::set_sum();
        core::slice::from_raw_parts(iov as *const IoVec, iovcnt)
    };
    let mut totol_len = 0usize;
    for (i, iov) in iovs.iter().enumerate() {
        if iov.len == 0 {
            continue;
        }
        log::debug!("[sys_writev]: iov[{}], ptr: {:#x}, len: {}", i, iov.base, iov.len);
        let buf = unsafe {
            Instruction::set_sum();
            core::slice::from_raw_parts(iov.base as *const u8, iov.len)
        };
        let write_len = file.write(buf).await;
        totol_len += write_len;
    }
    Ok(totol_len as isize)
}

/// at helper:
/// since many "xxxat" type file system syscalls will use the same logic of getting dentry,
/// we need to write a helper function to reduce code duplication
/// warning: for supporting more "at" syscall, emptry path is allowed here,
/// caller should check the path before calling at_helper if it doesnt expect empty path
pub fn at_helper(task: Arc<TaskControlBlock>, dirfd: isize, pathname: *const u8, flags: OpenFlags) -> Result<Arc<dyn Dentry>, SysError> {
    let _sum_guard = SumGuard::new();
    let dentry = match user_path_to_string(pathname) {
        Some(path) => {
            if path.starts_with("/") {
                global_find_dentry(&path)
            } else {
                // getting full path (absolute path)
                let fpath = if dirfd == AT_FDCWD {
                    // look up in the current dentry
                    let cw_dentry = task.with_cwd(|d| d.clone());
                    rel_path_to_abs(&cw_dentry.path(), &path).unwrap()
                } else {
                    // look up in the current task's fd table
                    // which the inode fd points to should be a dir
                    let dir = task.with_fd_table(|t| t.get_file(dirfd as usize))?;
                    let dentry = dir.dentry().unwrap();
                    rel_path_to_abs(&dentry.path(), &path).unwrap()
                };
                global_find_dentry(&fpath)
            }
        }
        None => {
            warn!("[at_helper]: using empty path!");
            if dirfd == AT_FDCWD {
                task.with_cwd(|d| d.clone())
            } else {
                let file = task.with_fd_table(|t| t.get_file(dirfd as usize))?;
                file.dentry().unwrap()
            }
        }
    };

    if flags.contains(OpenFlags::O_NOFOLLOW) {
        Ok(dentry)
    } else {
        let dentry = dentry.follow()?;
        Ok(dentry)
    }
}