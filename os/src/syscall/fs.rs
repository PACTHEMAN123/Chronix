//! File and filesystem-related syscalls
use core::{any::Any, cmp, ops::DerefMut, ptr::copy_nonoverlapping};

use alloc::{string::ToString, sync::Arc, vec};
use hal::{addr::{PhysAddrHal, PhysPageNumHal, VirtAddr, VirtAddrHal}, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::PageTableHal, println};
use log::{info, warn};
use strum::FromRepr;
use virtio_drivers::PAGE_SIZE;
use crate::{config::BLOCK_SIZE, drivers::BLOCK_DEVICE, fs::{
    get_filesystem, pipefs::make_pipe, vfs::{dentry::{self, global_find_dentry, global_update_dentry}, file::{open_file, SeekFrom}, fstype::MountFlags, inode::InodeMode, Dentry, DentryState, File}, AtFlags, Kstat, OpenFlags, RenameFlags, StatFs, UtsName, Xstat, XstatMask
}, mm::{translate_uva_checked, vm::{PageFaultAccessType, UserVmSpaceHal}, UserPtrRaw, UserSliceRaw}, processor::context::SumGuard, task::{fs::{FdFlags, FdInfo}, task::TaskControlBlock}, timer::{ffi::TimeSpec, get_current_time_duration}, utils::block_on};
use crate::utils::{
    path::*,
    string::*,
};
use super::{SysResult,SysError};
use crate::processor::processor::{current_processor,current_task,current_user_token};

/// syscall: write
pub async fn sys_write(fd: usize, buf: usize, len: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    log::debug!("task {} trying to write fd {}", task.gettid(), fd);
    let file = task.with_fd_table(|table| table.get_file(fd))?;
    let user_buf = 
        UserSliceRaw::new(buf as *mut u8, len)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
    let buf = user_buf.to_ref();
    let ret = file.write(buf).await?;

    // let start = buf & !(Constant::PAGE_SIZE - 1);
    // let end = buf + len;
    // let mut ret = 0;
    // for aligned_va in (start..end).step_by(Constant::PAGE_SIZE) {
    //     let va = aligned_va.max(buf);
    //     let len = (Constant::PAGE_SIZE - (va % Constant::PAGE_SIZE)).min(end - va);
    //     let va = VirtAddr::from(va);
    //     let pa = task.with_mut_vm_space(|vm| {
    //         translate_uva_checked(vm, va, PageFaultAccessType::READ).unwrap()
    //     });
    //     let data = pa.get_slice(len);
    //     ret += file.write(data).await?;
    // }

    return Ok(ret as isize);
}


/// syscall: read
pub async fn sys_read(fd: usize, buf: usize, len: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    // log::debug!("task {} trying to read fd {} to buf {:#x} with len {:#x}", task.gettid(), fd, buf, len);
    let file = task.with_fd_table(|table| table.get_file(fd))?;
    let user_buf = 
        UserSliceRaw::new(buf as *mut u8, len)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
    let buf = user_buf.to_mut();
    let ret = file.read(buf).await?;

    // let start = buf & !(Constant::PAGE_SIZE - 1);
    // let end = buf + len;
    // let mut ret = 0;
    // for aligned_va in (start..end).step_by(Constant::PAGE_SIZE) {
    //     let va = aligned_va.max(buf);
    //     let len = (Constant::PAGE_SIZE - (va % Constant::PAGE_SIZE)).min(end - va);
    //     let va = VirtAddr::from(va);
    //     let pa = task.with_mut_vm_space(|vm| {
    //         translate_uva_checked(vm, va, PageFaultAccessType::WRITE).unwrap()
    //     });
    //     let data = pa.get_slice_mut(len);
    //     let read_size = file.read(data).await?;
    //     ret += read_size;
    //     if read_size < len {
    //         break;
    //     }
    // }

    return Ok(ret as isize);
}

/// syscall: close
pub fn sys_close(fd: usize) -> SysResult {
    log::info!("[sys_close]: close on fd: {}", fd);
    let task = current_task().unwrap();
    task.with_mut_fd_table(|table| table.remove(fd))?;
    Ok(0)
}

/// syscall: lseek
pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> SysResult {
    #[derive(FromRepr)]
    #[repr(usize)]
    enum Whence {
        SeekSet = 0,
        SeekCur = 1,
        SeekEnd = 2,
        SeekData = 3,
        SeekHold = 4,
    }
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let whence = Whence::from_repr(whence).ok_or(SysError::EINVAL)?;
    let ret = match whence {
        Whence::SeekSet => file.seek(SeekFrom::Start(offset as u64))?,
        Whence::SeekCur => file.seek(SeekFrom::Current(offset as i64))?,
        Whence::SeekEnd => file.seek(SeekFrom::End(offset as i64))?,
        _ => {
            return Err(SysError::EINVAL)
        }
    };
    log::debug!("[sys_lseek]: ret: {}, file: {}", ret, fd);
    Ok(ret as isize)
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
    let task = current_task().unwrap();
    task.with_cwd(|cwd| {
        let path = cwd.path();
        if len < path.len() + 1 {
            info!("[sys_getcwd]: buf len too small to recv path");
            return Err(SysError::ERANGE);
        } else {
            //info!("copying path: {}, len: {}", path, path.len());
            let new_buf = UserSliceRaw::new(buf as *mut u8, len)
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
            new_buf.to_mut()[path.len()..].fill(0 as u8);
            new_buf.to_mut()[..path.len()].copy_from_slice(path.as_bytes());
            return Ok(buf as isize);
        }
    })
}

/// syscall: dup
pub fn sys_dup(old_fd: usize) -> SysResult {
    log::debug!("dup old fd: {}", old_fd);
    let task = current_task().unwrap();
    let new_fd = task.with_mut_fd_table(|table| table.dup_no_flag(old_fd))?;
    Ok(new_fd as isize)
}

/// syscall: dup3
pub fn sys_dup3(old_fd: usize, new_fd: usize, flags: u32) -> SysResult {
    log::debug!("dup3: old_fd = {}, new_fd = {}", old_fd, new_fd);
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
    let open_flags = OpenFlags::from_bits(flags as i32).unwrap();
    let at_flags = AtFlags::from_bits_truncate(flags as i32);
    let task = current_task().unwrap().clone();
    let path = user_path_to_string(
            UserPtrRaw::new(pathname), 
            &mut task.get_vm_space().lock()
    )?;
    // log::info!("task {} trying to open {}, oflags: {:?}, atflags: {:?}, dirfd {}", task.tid(), path, open_flags, at_flags, dirfd);
    let dentry = at_helper(task.clone(), dirfd, pathname, at_flags)?;
    if open_flags.contains(OpenFlags::O_CREAT) {
        // the dir may not exist
        if abs_path_to_name(&path).unwrap() != abs_path_to_name(&dentry.path()).unwrap() {
            return Err(SysError::ENOENT);
        }
        // inode not exist, create it as a regular file
        if open_flags.contains(OpenFlags::O_EXCL) && dentry.state() != DentryState::NEGATIVE {
            return Err(SysError::EEXIST);
        }
        let parent = dentry.parent().expect("[sys_openat]: can not open root as file!");
        let name = abs_path_to_name(&path).unwrap();
        let new_inode = parent.inode().unwrap().create(&name, InodeMode::FILE);
        match new_inode {
            Ok(inode) => {
                dentry.set_inode(inode);
            }
            Err(SysError::EEXIST) => {}
            _ => {
                panic!("should not reach here")
            }
        }
        
        // we shall not add child to parent until child is valid!
        parent.add_child(dentry.clone());
    }
    if dentry.state() == DentryState::NEGATIVE {
        log::warn!("cannot open {}, not exist", path);
        return Err(SysError::ENOENT);
    }
    let inode = dentry.inode().unwrap();
    if open_flags.contains(OpenFlags::O_DIRECTORY) && inode.inode_type() != InodeMode::DIR {
        return Err(SysError::ENOTDIR);
    }
    let file = dentry.open(open_flags).unwrap();
    file.set_flags(open_flags);
    let fd = task.with_mut_fd_table(|table| table.alloc_fd())?;
    let fd_info = FdInfo { file, flags: open_flags.into() };
    task.with_mut_fd_table(|t|t.put_file(fd, fd_info))?;
    log::info!("return fd {fd}");
    return Ok(fd as isize)
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
    let task = current_task().unwrap();
    let opt_path = user_path_to_string(
            UserPtrRaw::new(pathname), 
            &mut task.get_vm_space().lock()
        );
    if let Ok(path) = opt_path {
        let task = current_task().unwrap().clone();
        let dentry = at_helper(task, dirfd, pathname, AtFlags::empty())?;
        if dentry.state() != DentryState::NEGATIVE {
            return Err(SysError::EEXIST);
        }
        let parent = dentry.parent().unwrap();
        let name = abs_path_to_name(&path).unwrap();
        let new_inode = parent.inode().unwrap().create(&name, InodeMode::DIR).unwrap();
        dentry.set_inode(new_inode);
        dentry.set_state(DentryState::USED);
        parent.add_child(dentry);
    } else {
        warn!("[sys_mkdirat]: pathname is empty!");
        return Err(SysError::ENOENT);
    }
    Ok(0)
}

/// syscall: fstatat
pub fn sys_fstatat(dirfd: isize, pathname: *const u8, stat_buf: usize, flags: i32) -> SysResult {
    let _sum_guard= SumGuard::new();
    let at_flags = AtFlags::from_bits_truncate(flags);
    let o_flags = OpenFlags::from_bits_truncate(flags);
    log::debug!("fstatat dirfd {}, at_flags {:?}, oflags {:?}", dirfd, at_flags, o_flags);
    let task = current_task().unwrap().clone();
    let dentry = at_helper(task.clone(), dirfd, pathname, at_flags)?;
    log::debug!("fstatat dirfd {}, path {}, at_flags {:?}, oflags {:?}", dirfd, dentry.path(), at_flags, o_flags);
    let inode = dentry.inode();
    if inode.is_none() {
        return Err(SysError::ENOENT)
    }
    // debug_assert!(dentry.is_negative() == false);
    let stat = inode.unwrap().getattr();
    let stat_ptr = UserPtrRaw::new(stat_buf as *const Kstat)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    stat_ptr.write(stat);
    Ok(0)
}

/// chdir() changes the current working directory of the calling
/// process to the directory specified in path.
/// On success, zero is returned.  On error, -1 is returned, and errno
/// is set to indicate the error.
pub fn sys_chdir(path: *const u8) -> SysResult {
    let task = current_task().unwrap().clone();
    let path = user_path_to_string(
            UserPtrRaw::new(path), 
            &mut task.get_vm_space().lock()
        )?;
    info!("try to switch to path {}", path);
    let old_dentry = task.cwd();
    let new_dentry = if path.starts_with("/") {
        global_find_dentry(&path)?
    } else {
        old_dentry.find(&path)?.ok_or(SysError::ENOENT)?
    };
    if new_dentry.state() == DentryState::NEGATIVE {
        log::warn!("[sys_chdir]: dentry not found");
        return Err(SysError::ENOENT);
    } else if !new_dentry.inode().unwrap().inode_type().contains(InodeMode::DIR) {
        log::warn!("[sys_chdir]: path is not dir");
        return Err(SysError::ENOTDIR);
    } else {
        let task = current_task().unwrap().clone();
        task.set_cwd(new_dentry);
        return Ok(0);
    }
}

/// The fchdir() function shall be equivalent to chdir() except that
/// the directory that is to be the new current working directory is
/// specified by the file descriptor fildes.
pub fn sys_fchdir(fd: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let dir = task.with_fd_table(|t| t.get_file(fd))?;
    let dentry = dir.dentry().unwrap();
    let inode = dentry.inode().unwrap();
    if inode.inode_type() != InodeMode::DIR {
        return Err(SysError::ENOTDIR);
    }
    log::info!("[fchdir]: task {} change cwd to {}", task.tid(), dentry.path());
    task.set_cwd(dentry);
    Ok(0)
}


const PIPE_BUF_LEN: usize = 16 * PAGE_SIZE;
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
    let read_fd = task.with_mut_fd_table(|t|t.alloc_fd())?;
    task.with_mut_fd_table(|t| t.put_file(read_fd, FdInfo { file: read_file, flags: flags.into() }))?;
    let write_fd = task.with_mut_fd_table(|t|t.alloc_fd())?;
    task.with_mut_fd_table(|t| t.put_file(write_fd, FdInfo { file: write_file, flags: flags.into() }))?;

    let _sum = SumGuard::new();
    // let pipefd = unsafe { core::slice::from_raw_parts_mut(pipe, 2 * core::mem::size_of::<i32>()) };
    let pipefd = UserSliceRaw::new(pipe, 2)
        .ensure_write(&mut task.vm_space.lock())
        .ok_or(SysError::EFAULT)?;
    info!("read fd: {}, write fd: {}", read_fd, write_fd);
    pipefd.to_mut()[0] = read_fd as i32;
    pipefd.to_mut()[1] = write_fd as i32;
    Ok(0)
}

/// syscall fstat
pub fn sys_fstat(fd: usize, stat_buf: usize) -> SysResult {
    let _sum_guard = SumGuard::new();
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let stat = file.inode().unwrap().getattr();
    log::debug!("[sys_fstat]: fstat file {}, size {}", fd, stat.st_size);
    let stat_ptr = stat_buf as *mut Kstat;
    unsafe {
        let _sum_guard = SumGuard::new();
        *stat_ptr = stat;
    }
    return Ok(0);
}

/// syscall statfs
/// TODO
pub fn sys_statfs(_path: usize, buf: usize) -> SysResult {
    let info = StatFs {
        f_type: 0x2011BAB0 as i64,
        f_bsize: BLOCK_SIZE as i64,
        f_blocks: 1 << 27,
        f_bfree: 1 << 26,
        f_bavail: 1 << 20,
        f_files: 1 << 10,
        f_ffree: 1 << 9,
        f_fsid: [0; 2],
        f_namelen: 1 << 8,
        f_frsize: 1 << 9,
        f_flags: 1 << 1 as i64,
        f_spare: [0; 4],
    };
    unsafe {
        Instruction::set_sum();
        (buf as *mut StatFs).write(info);
    }
    Ok(0)
}

/// syscall statx
pub fn sys_statx(dirfd: isize, pathname: *const u8, flags: i32, mask: u32, statx_buf: VirtAddr) -> SysResult {
    let _sum_guard = SumGuard::new();
    let open_flags = OpenFlags::from_bits(flags).ok_or(SysError::EINVAL)?;
    let at_flags = AtFlags::from_bits_truncate(flags);
    let mask = XstatMask::from_bits_truncate(mask);
    let task = current_task().unwrap().clone();
    let path = UserPtrRaw::new(pathname)
        .cstr_slice(&mut task.vm_space.lock())?
        .to_str()
        .unwrap()
        .to_string();

    log::debug!("[sys_statx]: statx dirfd: {}, path: {}, at_flags {:?}, open_flags: {:?}", dirfd, path, at_flags, open_flags);

    let dentry = at_helper(task.clone(), dirfd, pathname, at_flags)?;
    if dentry.state() == DentryState::NEGATIVE && dentry.inode().is_none() {
        return Err(SysError::ENOENT);
    }
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

/// syscall: syslog
/// TODO: unimplement
pub fn sys_syslog(_log_type: usize, _bufp: usize, _len: usize) -> SysResult {
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
    let user_buf = UserSliceRaw::new(buf as *mut u8, len)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EINVAL)?;
    let buf_slice = user_buf.to_mut();
    assert!(buf_slice.len() == len);

    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let dentry = file.dentry().unwrap();
    let mut buf_it = buf_slice;
    let mut writen_len = 0;
    for child in dentry.load_child_dentry()?.iter().skip(file.pos()) {
        assert!(child.state() != DentryState::NEGATIVE);
        // align to 8 bytes
        let c_name_len = child.name().len() + 1;
        let rec_len = (LEN_BEFORE_NAME + c_name_len + 7) & !0x7;
        let inode = child.inode().unwrap();
        let linux_dirent = LinuxDirent64 {
            d_ino: inode.inode_inner().ino as u64,
            d_off: file.pos() as u64,
            d_type: inode.inode_inner().mode().bits() as u8,
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
    log::debug!("writen_len: {}", writen_len);
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
    const AT_REMOVEDIR: i32 = 0x200;
    let task = current_task().unwrap().clone();
    let path = user_path_to_string(
            UserPtrRaw::new(pathname), 
            &mut task.get_vm_space().lock()
        )?;
    log::info!("[sys_unlinkat]: task {} unlink {}", task.tid(), path);
    let dentry = at_helper(task, dirfd, pathname, AtFlags::AT_SYMLINK_NOFOLLOW)?;
    if dentry.parent().is_none() {
        warn!("cannot unlink root!");
        return Err(SysError::ENOENT);
    }
    if dentry.is_negative() {
        return Err(SysError::ENOENT);
    }
    let inode = dentry.inode().unwrap();
    let inode_mode = inode.inode_inner().mode();
    let is_dir = inode_mode == InodeMode::DIR;
    if flags == AT_REMOVEDIR && !is_dir {
        return Err(SysError::ENOTDIR);
    } else if flags != AT_REMOVEDIR && is_dir {
        // return Err(SysError::EPERM);
    }
    // should clear inode first to drop inode (flush datas to disk)
    dentry.clear_inode();
    inode.clean_cached();
    drop(inode);
    // use parent inode to remove the inode in the fs
    let name = abs_path_to_name(&path).unwrap();
    let parent = dentry.parent().unwrap();
    parent.inode().unwrap().remove(&name, inode_mode).expect("remove failed");
    parent.remove_child(&name);

    //inode.unlink().expect("inode unlink failed");
    Ok(0)
}

/// syscall: symlinkat
pub fn sys_symlinkat(old_path_ptr: *const u8, new_dirfd: isize, new_path_ptr: *const u8) -> SysResult {
    let task = current_task().unwrap().clone();
    let old_path = user_path_to_string(
        UserPtrRaw::new(old_path_ptr), 
        &mut task.get_vm_space().lock())?;
    let new_path = user_path_to_string(
        UserPtrRaw::new(new_path_ptr), 
        &mut task.get_vm_space().lock())?;
    if old_path.is_empty() || new_path.is_empty() {
        return Err(SysError::ENOENT);
    }
    log::info!("[sys_symlinkat] task {}, sym-link old path {} to new path {}, fd {new_dirfd}", task.tid(), old_path, new_path);
    let new_dentry = at_helper(task, new_dirfd, new_path_ptr, AtFlags::AT_SYMLINK_NOFOLLOW)?;
    let new_path = new_dentry.path();
    let parent = new_dentry.parent().unwrap().inode().unwrap();
    // let old_path = old_dentry.path();
    let new_inode = parent.symlink(&old_path, &new_path)?;
    if new_dentry.inode().is_some() && !new_dentry.is_negative() {
        return Err(SysError::EEXIST);
    } else {
        log::info!("create a new symlink");
        new_dentry.set_inode(new_inode);
    }
    // global_update_dentry(&new_path, new_inode)?;
    Ok(0)
}

/// Modify the permissions of a file or directory relative to a certain
/// directory or location
pub fn sys_fchmodat(dirfd: isize, pathname: *const u8, mode: u32, flags: i32) -> SysResult {
    let task = current_task().unwrap().clone();
    let mode = InodeMode::from_bits_truncate(mode);
    let at_flags = AtFlags::from_bits_truncate(flags);
    let path = UserPtrRaw::new(pathname)
        .cstr_slice(&mut task.vm_space.lock())?
        .to_str()
        .unwrap()
        .to_string();
    log::info!("[sys_fchmodat] task {} change {} mode to {:?}, flags", task.tid(), path, mode);
    let dentry = at_helper(task, dirfd, pathname, at_flags)?;
    if dentry.is_negative() && dentry.inode().is_none() {
        return Err(SysError::ENOENT);
    }
    let inode = dentry.inode().unwrap();
    let inode_type = inode.inode_type();
    inode.inode_inner().set_mode(mode | inode_type);
    Ok(0)
}

/// The fchmod() function shall be equivalent to chmod() except that
/// the file whose permissions are changed is specified by the file
/// descriptor fildes.
pub fn sys_fchmod(fd: isize, mode: u32) -> SysResult {
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd as usize))?;
    let mode = InodeMode::from_bits_truncate(mode);
    let inode = file.inode().unwrap();
    let inode_type = inode.inode_type();
    inode.inode_inner().set_mode(mode | inode_type);
    Ok(0)
}

/// change the owner and group of a file
pub fn sys_fchownat(dirfd: isize, pathname: *const u8, uid: i32, gid: i32, flags: i32) -> SysResult {
    let task = current_task().unwrap().clone();
    let at_flags = AtFlags::from_bits_truncate(flags);
    log::info!("[fchownat] at_flags {:?} flags {:#x}", at_flags, flags);
    if flags != 0 && !at_flags
    .intersection(!(AtFlags::AT_EMPTY_PATH | AtFlags::AT_SYMLINK_NOFOLLOW)).is_empty() {
        return Err(SysError::EINVAL);
    }
    let path = user_path_to_string(
        UserPtrRaw::new(pathname), 
        &mut task.get_vm_space().lock())?;
    log::info!("[sys_fchownat] path {} owner {}, gid {}", path, uid, gid);
    let dentry = at_helper(task, dirfd, pathname, at_flags)?;
    if dentry.is_negative() && dentry.inode().is_none() {
        return Err(SysError::ENOENT);
    }
    let inode = dentry.inode().unwrap();
    if gid != -1 {
        inode.inode_inner().set_gid(gid as u32);
    }
    if uid != -1 {
        inode.inode_inner().set_uid(uid as u32);
    }
    let old_mode = inode.inode_inner().mode();
    let inode_type = old_mode.get_type();
    let new_mode = if !old_mode.contains(InodeMode::GROUP_EXEC) {
        old_mode.intersection(!InodeMode::SET_UID)
    } else {
        old_mode.intersection(!(InodeMode::SET_GID | InodeMode::SET_UID))
    };
    inode.inode_inner().set_mode(new_mode | inode_type);
    Ok(0)
}


pub fn sys_fchown(fd: isize, uid: i32, gid: i32) -> SysResult {
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd as usize))?;
    let inode = file.inode().unwrap();
    if gid != -1 {
        inode.inode_inner().set_gid(gid as u32);
    }
    if uid != -1 {
        inode.inode_inner().set_uid(uid as u32);
    }
    let old_mode = inode.inode_inner().mode();
    let inode_type = old_mode.get_type();
    let new_mode = if !old_mode.contains(InodeMode::GROUP_EXEC) {
        old_mode.intersection(!InodeMode::SET_UID)
    } else {
        old_mode.intersection(!(InodeMode::SET_GID | InodeMode::SET_UID))
    };
    inode.inode_inner().set_mode(new_mode | inode_type);
    Ok(0)
}

/// syscall: readlinkat
/// does not append
/// a terminating null byte to buf.  It will (silently) truncate the
/// contents (to a length of bufsiz characters), in case the buffer is
/// too small to hold all of the contents.
pub fn sys_readlinkat(dirfd: isize, pathname: *const u8, buf: usize, len: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let dentry = at_helper(task.clone(), dirfd, pathname, AtFlags::AT_SYMLINK_NOFOLLOW)?;
    info!("[sys_readlinkat]: reading link {}", dentry.path());
    if dentry.state() == DentryState::NEGATIVE {
        return Err(SysError::EBADF);
    }
    let inode = dentry.inode().ok_or(SysError::ENOENT)?;
    if inode.inode_inner().mode() != InodeMode::LINK {
        return Err(SysError::EINVAL);
    }
    
    let path = inode.readlink()?;
    if len as isize <= 0 {
        return Err(SysError::EINVAL);
    }
    let new_buf = UserSliceRaw::new(buf as *mut u8, len)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EINVAL)?;
    if len > path.len() {
        new_buf.to_mut()[path.len()..].fill(0u8);
    }
    let copy_size = cmp::min(len, path.len());
    new_buf.to_mut()[..copy_size].copy_from_slice(&path.as_bytes()[..copy_size]);
    return Ok(path.len() as isize)
}

/// syscall: utimensat
/// The utime() system call changes the access and modification times
/// of the inode specified by filename to the actime and modtime
/// fields of times respectively.  The status change time (ctime) will
/// be set to the current time, even if the other time stamps don't
/// actually change.
pub fn sys_utimensat(dirfd: isize, pathname: *const u8, times: usize, flags: i32) -> SysResult {
    const UTIME_NOW: usize = 0x3fffffff;
    const UTIME_OMIT: usize = 0x3ffffffe;
    let task = current_task().unwrap().clone();
    let at_flags = AtFlags::from_bits_truncate(flags);
    log::info!("[sys_utimensat]: dirfd {}, pathname ptr {:#x}, flags {:?}", dirfd, pathname as usize, at_flags);
    if dirfd as i32 != AtFlags::AT_FDCWD.bits() && pathname.is_null() && at_flags.contains(AtFlags::AT_SYMLINK_NOFOLLOW) {
        return Err(SysError::EINVAL)
    }
    // VERSIONS: .... To support this, the Linux
    // utimensat() system call implements a nonstandard feature: if
    // pathname is NULL, then the call modifies the timestamps of the
    // file referred to by the file descriptor dirfd
    let inode = if pathname.is_null() {
        let file = task.with_fd_table(|t| t.get_file(dirfd as usize))?;
        file.inode().unwrap()
    } else {
        let dentry = at_helper(task.clone(), dirfd, pathname, at_flags)?;
        log::info!("[sys_utimensat]: path: {}", dentry.path());
        if dentry.is_negative() {
            return Err(SysError::ENOENT);
        }
        dentry.inode().unwrap()
    };
    
    let inner = inode.inode_inner();
    
    let current_time = TimeSpec::from(get_current_time_duration());
    if times == 0 {
        inner.set_atime(current_time);
        inner.set_ctime(current_time);
        inner.set_mtime(current_time);
    } else {
        let times_ptr =
            UserSliceRaw::new(times as *mut TimeSpec, 2)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        let times = times_ptr.to_mut();
        log::info!("[sys_utimensat] times {:?}", times);
        match times[0].tv_nsec {
            UTIME_NOW => inner.set_atime(current_time),
            UTIME_OMIT => {}
            _ => inner.set_atime(times[0]),
        }
        match times[1].tv_nsec {
            UTIME_NOW => *inner.mtime.lock() = current_time,
            UTIME_OMIT => {}
            _ => inner.set_mtime(times[1]),
        }
        inner.set_ctime(current_time);
    }
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
    file.ioctl(cmd, arg)
}

/* torvalds/linux/include/uapi/asm-generic/fcntl.h */
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
    F_GETLK	= 5,
    F_SETLK	= 6,
    F_SETLKW = 7,
    F_SETOWN = 8,	/* for sockets. */
    F_GETOWN = 9,	/* for sockets. */
    F_SETSIG = 10,	/* for sockets. */
    F_GETSIG = 11,	/* for sockets. */
    F_GETLK64 = 12,	/*  using 'struct flock64' */
    F_SETLK64 = 13,
    F_SETLKW64 = 14,
    F_SETOWN_EX = 15,
    F_GETOWN_EX	= 16,
    F_GETOWNER_UIDS	= 17,
    #[default]
    F_UNIMPL,
}

/// syscall: fcntl
pub fn sys_fnctl(fd: usize, op: isize, arg: usize) -> SysResult {
    let op = FcntlOp::from_repr(op).unwrap_or_default();
    let task = current_task().unwrap().clone();
    log::info!("[fcntl] op {:?}", op);
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
            let new_fd = task.with_mut_fd_table(|t| t.dup_with_bound(fd, arg, OpenFlags::O_CLOEXEC.into()))?;
            Ok(new_fd as isize)
        }
        FcntlOp::F_GETFD => {
            // Return (as the function result) the file descriptor flags;
            // arg is ignored.
            let fd_info = task.with_fd_table(|t| t.get_fd_info(fd))?;
            Ok(fd_info.flags().bits() as isize)
        }
        FcntlOp::F_SETFD => {
            let fd_flags = FdFlags::from_bits_truncate(arg as u8);
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
            let mask = OpenFlags::O_APPEND | OpenFlags::O_ASYNC | 
                OpenFlags::O_DIRECT | OpenFlags::O_NOATIME | OpenFlags::O_NONBLOCK;
            log::info!("set flags {:?}, {:#x} ,raw flags {:#x}", flags, flags.bits(), arg);
            let file = task.with_fd_table(|table| table.get_file(fd))?;
            let old_flags = file.flags();
            file.set_flags(old_flags.masked_set_flags(flags, mask));
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
    if (iovcnt as isize) < 0 {
        return Err(SysError::EINVAL);
    }
    let iovs = UserSliceRaw::new(iov as *const IoVec, iovcnt)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let mut totol_len = 0usize;
    // let mut offset = file.pos();
    for (i, iov) in iovs.to_ref().iter().enumerate() {
        if iov.len == 0 {
            continue;
        }
        log::debug!("[sys_readv]: iov[{}], ptr: {:#x}, len: {}, read from file pos {}", i, iov.base, iov.len, file.pos());
        
        let iov_buf =
            UserSliceRaw::new(iov.base as *mut u8, iov.len)
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
        let ret = file.read(iov_buf.to_mut()).await?;

        // ugly way
        // let start = iov.base & !(Constant::PAGE_SIZE - 1);
        // let end = iov.base + iov.len;
        // let mut ret = 0;

        // for aligned_va in (start..end).step_by(Constant::PAGE_SIZE) {
        //     let va = aligned_va.max(iov.base);
        //     let len = (Constant::PAGE_SIZE - (va % Constant::PAGE_SIZE)).min(end - va);
        //     let va = VirtAddr::from(va);
        //     let pa = task.with_mut_vm_space(|vm| {
        //         translate_uva_checked(vm, va, PageFaultAccessType::WRITE).unwrap()
        //     });
        //     let data = pa.get_slice_mut(len);
        //     let read_size = file.read(data).await?;
        //     ret += read_size;
        //     if read_size < len {
        //         break;
        //     }
        // }

        totol_len += ret;
        // offset += ret;
    }
    // assert!(offset == file.pos());
    Ok(totol_len as isize)
}

/// The writev() function shall be equivalent to write(), except as
/// described below. The writev() function shall gather output data
/// from the iovcnt buffers specified by the members of the iov array:
/// iov[0], iov[1], ..., iov[iovcnt-1].
pub async fn sys_writev(fd: usize, iov: usize, iovcnt: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let iovs = UserSliceRaw::new(iov as *const IoVec, iovcnt)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EINVAL)?;
    let mut totol_len = 0usize;
    for (i, iov) in iovs.to_ref().iter().enumerate() {
        if iov.len == 0 {
            continue;
        }
        log::debug!("[sys_writev]: iov[{}], ptr: {:#x}, len: {}, file pos {}", i, iov.base, iov.len, file.pos());

        let iov_buf =
            UserSliceRaw::new(iov.base as *mut u8, iov.len)
                .ensure_read(&mut task.get_vm_space().lock())
                .ok_or(SysError::EINVAL)?;
        let ret = file.write(iov_buf.to_ref()).await?;

        // let start = iov.base & !(Constant::PAGE_SIZE - 1);
        // let end = iov.base + iov.len;
        // let mut ret = 0;
        // for aligned_va in (start..end).step_by(Constant::PAGE_SIZE) {
        //     let va = aligned_va.max(iov.base);
        //     let len = (Constant::PAGE_SIZE - (va % Constant::PAGE_SIZE)).min(end - va);
        //     let va = VirtAddr::from(va);
        //     let pa = task.with_mut_vm_space(|vm| {
        //         translate_uva_checked(vm, va, PageFaultAccessType::READ).unwrap()
        //     });
        //     let data = pa.get_slice(len);
        //     ret += file.write(data).await?;
        // }
        totol_len += ret;
    }
    Ok(totol_len as isize)
}

/// pread() reads up to count bytes from file descriptor fd at offset
/// offset (from the start of the file) into the buffer starting at buf.  
/// The file offset is not changed.
/// TODOS: now mostly copy from sys_read; need to deal with file offset under multi-thread
/// UGLY: now reset file offset, try to add a read_at method for file?
pub async fn sys_pread(fd: usize, buf: usize, count: usize, offset: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    log::debug!("[sys_pread] task {} try to read fd {} to buf {:#x} at offset {}, len {}", task.tid(), fd, buf, offset, count);
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let old_pos = file.pos();
    // assume: during file read, no task switch
    file.seek(SeekFrom::Start(offset as u64))?;
    let user_buf =
        UserSliceRaw::new(buf as *mut u8, count)
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
    let ret = file.read(user_buf.to_mut()).await?;
    // let start = buf & !(Constant::PAGE_SIZE - 1);
    // let end = buf + count;
    // let mut ret = 0;
    // for aligned_va in (start..end).step_by(Constant::PAGE_SIZE) {
    //     let va = aligned_va.max(buf);
    //     let len = (Constant::PAGE_SIZE - (va % Constant::PAGE_SIZE)).min(end - va);
    //     let va = VirtAddr::from(va);
    //     let pa = task.with_mut_vm_space(|vm| {
    //         translate_uva_checked(vm, va, PageFaultAccessType::WRITE).unwrap()
    //     });
    //     let data = pa.get_slice_mut(len);
    //     let read_size = file.read(data).await?;
    //     ret += read_size;
    //     if read_size < len {
    //         break;
    //     }
    // }
    file.set_pos(old_pos);
    Ok(ret as isize)
}

/// pwrite() writes up to count bytes from the buffer starting at buf 
/// to the file descriptor fd at offset offset. 
/// The file offset is not changed.
/// NOTICE: see pread
pub async fn sys_pwrite(fd: usize, buf: usize, count: usize, offset: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    log::debug!("[sys_pwrite] task {} try to read fd {} to buf {:#x} at offset {}, len {}", task.tid(), fd, buf, offset, count);
    let file = task.with_fd_table(|t| t.get_file(fd))?;
    let old_pos = file.pos();
    // assume: during file read, no task switch
    file.seek(SeekFrom::Start(offset as u64))?;
    let user_buf = 
        UserSliceRaw::new(buf as *mut u8, count)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EINVAL)?;
    let ret = file.write(user_buf.to_ref()).await?;
    // let start = buf & !(Constant::PAGE_SIZE - 1);
    // let end = buf + count;
    // let mut ret = 0;
    // for aligned_va in (start..end).step_by(Constant::PAGE_SIZE) {
    //     let va = aligned_va.max(buf);
    //     let len = (Constant::PAGE_SIZE - (va % Constant::PAGE_SIZE)).min(end - va);
    //     let va = VirtAddr::from(va);
    //     let pa = task.with_mut_vm_space(|vm| {
    //         translate_uva_checked(vm, va, PageFaultAccessType::READ).unwrap()
    //     });
    //     let data = pa.get_slice(len);
    //     ret += file.write(data).await?;
    // }
    file.set_pos(old_pos);
    log::debug!("finish pwrite return {}", ret);
    Ok(ret as isize)
}

/// sendfile() copies data between one file descriptor and another.
/// If offset is not NULL, then it points to a variable holding the
/// file offset from which sendfile() will start reading data from
/// in_fd.  When sendfile() returns, this variable will be set to the
/// offset of the byte following the last byte that was read. 
/// 
/// If offset is not NULL, then sendfile() does not modify the file
/// offset of in_fd; otherwise the file offset is adjusted to reflect
/// the number of bytes read from in_fd.
/// 
/// If offset is NULL, then data will be read from in_fd starting at
/// the file offset, and the file offset will be updated by the call.
pub async fn sys_sendfile(out_fd: usize, in_fd: usize, offset: usize, count: usize) -> SysResult {
    info!("[sys_sendfile]: out fd: {out_fd}, in fd: {in_fd}, offset: {offset}, count: {:#x}", count);
    let task = current_task().unwrap().clone();
    let in_file = task.with_fd_table(|t| t.get_file(in_fd))?;
    let out_file = task.with_fd_table(|t| t.get_file(out_fd))?;
    let mut buf = vec![0u8; count];
    let off_ptr = {
        UserPtrRaw::new(offset as *mut usize)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
    };
    let len;
    if off_ptr.raw == core::ptr::null() {
        len = in_file.read(&mut buf).await?;
    } else {
        let off = off_ptr.to_mut();
        if (*off as isize) < 0 {
            return Err(SysError::EINVAL);
        }
        len = in_file.read_at(*off, &mut buf).await?;
        *off += len;
    }
    let ret = out_file.write(&buf[..len]).await?;
    Ok(ret as isize)
}

/// syscall: linkat
/// link() creates a new link (also known as a hard link) to an existing file.
/// The linkat() system call operates in exactly the same way as link(2), 
pub fn sys_linkat(old_dirfd: isize, old_pathname: *const u8, new_dirfd: isize, new_pathname: *const u8, flags: i32) -> SysResult {
    let task = current_task().unwrap().clone();
    let at_flags = AtFlags::from_bits_truncate(flags);
    let old_dentry = at_helper(task.clone(), old_dirfd, old_pathname, at_flags)?;
    let new_dentry = at_helper(task.clone(), new_dirfd, new_pathname, at_flags)?;
    log::debug!("[sys_linkat]: try to create hard link between {} {}", old_dentry.path(), new_dentry.path());
    let old_inode = old_dentry.inode().ok_or(SysError::ENOENT)?;
    old_inode.link(&new_dentry.path())?;
    new_dentry.set_inode(old_inode);
    new_dentry.set_state(DentryState::USED);
    Ok(0)
}

pub const F_OK: i32 = 0;
pub const X_OK: i32 = 1;
pub const W_OK: i32 = 2;
pub const R_OK: i32 = 4;
/// syscall: faccessat
/// access() checks whether the calling process can access the file
/// pathname.  If pathname is a symbolic link, it is dereferenced.
/// TODO: now do nothing
pub fn sys_faccessat(dirfd: isize, pathname: *const u8, _mode: usize, flags: i32) -> SysResult {
    let at_flags = AtFlags::from_bits_truncate(flags);
    let task = current_task().unwrap().clone();
    let _ = user_path_to_string(
            UserPtrRaw::new(pathname), 
            &mut task.get_vm_space().lock()
    )?;

    let task = current_task().unwrap().clone();
    let dentry = at_helper(task, dirfd, pathname, at_flags)?;
    if dentry.is_negative() {
        return Err(SysError::ENOENT);
    }
    Ok(0)
}

// Test access permitted for effective IDs, not real IDs.
pub const AT_EACCESS: i32 = 0x200;
/// 
pub fn sys_faccessat2(dirfd: isize, pathname: *const u8, mode: i32, flags: i32) -> SysResult {
    if flags != 0 && flags & !(AT_EACCESS | (AtFlags::AT_EMPTY_PATH | AtFlags::AT_SYMLINK_NOFOLLOW).bits()) != 0 
    {
        return Err(SysError::EINVAL);
    }
    if mode != F_OK && mode & !(X_OK | W_OK | R_OK) != 0
    {
        return Err(SysError::EINVAL);
    }
    let task = current_task().unwrap().clone();
    let path = user_path_to_string(
            UserPtrRaw::new(pathname), 
            &mut task.get_vm_space().lock()
    )?;

    let at_flags = AtFlags::from_bits_truncate(flags);
    log::info!("path {} at_flags: {:?}", path, at_flags);
    let task = current_task().unwrap().clone();
    let dentry = at_helper(task, dirfd, pathname, at_flags)?;
    if dentry.is_negative() {
        return Err(SysError::ENOENT);
    }
    Ok(0)
}

/// rename() renames a file, moving it between directories if
/// required.  Any other hard links to the file (as created using
/// link(2)) are unaffected.  Open file descriptors for oldpath are
/// also unaffected.
/// renameat2() has an additional flags argument.  A renameat2() call
/// with a zero flags argument is equivalent to renameat().
pub fn sys_renameat2(old_dirfd: isize, old_path: *const u8, new_dirfd: isize, new_path: *const u8, flags: i32) -> Result<isize, SysError> {
    let task = current_task().unwrap().clone();
    let flags = RenameFlags::from_bits(flags).ok_or(SysError::EINVAL)?;
    let old_dentry = at_helper(task.clone(), old_dirfd, old_path, AtFlags::AT_SYMLINK_NOFOLLOW)?;
    let new_dentry = at_helper(task.clone(), new_dirfd, new_path, AtFlags::AT_SYMLINK_NOFOLLOW)?;

    if flags.contains(RenameFlags::RENAME_EXCHANGE)
            && (flags.contains(RenameFlags::RENAME_NOREPLACE)
                || flags.contains(RenameFlags::RENAME_WHITEOUT))
    {
        return Err(SysError::EINVAL);
    }
    // the new dentry can not be the descendant of the old dentry
    let mut parent_opt = new_dentry.parent();
    while let Some(parent) = parent_opt {
        if Arc::ptr_eq(&parent, &old_dentry) {
            return Err(SysError::EINVAL);
        }
        parent_opt = parent.parent();
    }

    if new_dentry.state() == DentryState::NEGATIVE && flags.contains(RenameFlags::RENAME_EXCHANGE) {
        return Err(SysError::ENOENT);
    } else if flags.contains(RenameFlags::RENAME_NOREPLACE) {
        return Err(SysError::EEXIST);
    }

    let old_inode = old_dentry.inode().ok_or(SysError::ENOENT)?;
    let new_inode = new_dentry.inode();
    old_inode.rename(&new_dentry.path(), new_inode)?;
    new_dentry.set_inode(old_inode);
    // warning: due to lwext4 unsupport for RENAME_EXCHANGE
    if flags.contains(RenameFlags::RENAME_EXCHANGE) {
        old_dentry.set_inode(new_dentry.inode().unwrap());
    } else {
        old_dentry.clear_inode();
    }
    Ok(0)
}

/// syscall: ftruncate
pub fn sys_ftruncate(fildes: usize, length: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let file = task.with_fd_table(|f| f.get_file(fildes))?;
    log::info!("[sys_ftruncate] fd {} truncate size to {}", fildes, length);
    file.inode().unwrap().truncate(length)?;
    Ok(0)
}

/// fake async
pub fn sys_fdatasync(fd: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let _ = task.with_fd_table(|t| t.get_file(fd))?;
    Ok(0)
}


/// at helper:
/// since many "xxxat" type file system syscalls will use the same logic of getting dentry,
/// we need to write a helper function to reduce code duplication
/// warning: for supporting more "at" syscall, emptry path is allowed here,
/// caller should check the path before calling at_helper if it doesnt expect empty path
pub fn at_helper(task: Arc<TaskControlBlock>, dirfd: isize, pathname: *const u8, flags: AtFlags) -> Result<Arc<dyn Dentry>, SysError> {
    let _sum_guard = SumGuard::new();
    if pathname.is_null() && !flags.contains(AtFlags::AT_EMPTY_PATH) {
        return Err(SysError::EFAULT);
    }
    let path = user_path_to_string(
            UserPtrRaw::new(pathname), 
            &mut task.get_vm_space().lock()
    )?;
    at_helper1(task, dirfd, &path, flags)
}

pub fn at_helper1(task: Arc<TaskControlBlock>, dirfd: isize, path: &str, flags: AtFlags) -> Result<Arc<dyn Dentry>, SysError> {
    let dentry = if path != "" {
        if path.starts_with("/") {
                global_find_dentry(&path)?
        } else {
            // getting full path (absolute path)
            let mut rel_path = path.to_string();
            let mut parent_dentry = if dirfd as i32 == AtFlags::AT_FDCWD.bits() {
                // look up in the current dentry
                task.with_cwd(|d| d.clone())
            } else {
                // look up in the current task's fd table
                // which the inode fd points to should be a dir
                let dir = task.with_fd_table(|t| t.get_file(dirfd as usize))?;
                dir.dentry().unwrap()
            };
            if parent_dentry.is_negative() {
                return Err(SysError::ENOENT)
            }

            // resolve ../ to get final parent
            // Symbolic links may contain ..  path components, which (if used at
            // the start of the link) refer to the parent directories of that in
            // which the link resides.
            let mut cnt = 0usize;
            while rel_path.starts_with("../") {
                info!("before resolve parent {}, rel_path {}", parent_dentry.path(), rel_path);
                if cnt > 0 {
                    parent_dentry = parent_dentry.parent().expect("failed no parent");
                }
                rel_path = rel_path.trim_start_matches("../").to_string();
                cnt += 1;
                if cnt >= 40 {
                    warn!("should not have so many ../, may occur some error");
                }
                info!("finish resolve parent {}, rel_path {}", parent_dentry.path(), rel_path);
            }

            let fpath = rel_path_to_abs(&parent_dentry.path(), &rel_path).unwrap();

            global_find_dentry(&fpath)?
        }
    } else {
        if !flags.contains(AtFlags::AT_EMPTY_PATH) {
                return Err(SysError::ENOENT);
        }
        if dirfd as i32 == AtFlags::AT_FDCWD.bits() {
            task.with_cwd(|d| d.clone())
        } else {
            let file = task.with_fd_table(|t| t.get_file(dirfd as usize))?;
            file.dentry().unwrap()
        }
    };

    if flags.contains(AtFlags::AT_SYMLINK_NOFOLLOW) {
        Ok(dentry)
    } else {
        let dentry = dentry.follow(task, dirfd, flags)?;
        Ok(dentry)
    }
}


/// umask() sets the calling process's file mode creation mask (umask) to
/// mask & 0777 
pub fn sys_umask(_mask: i32) -> SysResult {
    // TODO: implement this
    Ok(0x777)
}