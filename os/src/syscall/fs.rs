//! File and filesystem-related syscalls
use alloc::string::ToString;
use log::{info, warn};
use virtio_drivers::PAGE_SIZE;

use crate::{fs::{
    ext4::open_file, pipe::make_pipe, vfs::{dentry::{self, global_find_dentry}, inode::InodeMode, DentryState, File}, Kstat, OpenFlags, UtsName, AT_FDCWD, AT_REMOVEDIR
}, processor::context::SumGuard};
use crate::utils::{
    path::*,
    string::*,
};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::processor::processor::{current_processor,current_task,current_user_token};

/// syscall: write
pub async fn sys_write(fd: usize, buf: usize, len: usize) -> isize {
    let token = current_user_token(&current_processor());
    let task = current_task().unwrap().clone();
    //info!("task {} trying to write fd {}", task.gettid(), fd);
    let table_len = task.with_fd_table(|table|table.len());
    if fd >= table_len {
        return -1;
    }
    if let Some(file) = task.with_fd_table(|table| table[fd].clone()) {
        // info!("write to file");
        if !file.writable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        let ret = file.write(UserBuffer::new(translated_byte_buffer(token, buf as *const u8, len))).await;
        ret as isize
    } else {
        -1
    }
}


/// syscall: read
pub async fn sys_read(fd: usize, buf: usize, len: usize) -> isize {
    let token = current_user_token(&current_processor());
    let task = current_task().unwrap().clone();
    //info!("task {} trying to read fd {}", task.gettid(), fd);
    let table_len = task.with_fd_table(|table|table.len());
    if fd >= table_len{
        return -1;
    }
    if let Some(file) = task.with_fd_table(|table| table[fd].clone()) {
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        //drop(inner);
        let ret = file.read(UserBuffer::new(translated_byte_buffer(token, buf as *const u8, len))).await;
        ret as isize
    } else {
        -1
    }
}

/// syscall: close
pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let table_len = task.with_fd_table(|table|table.len());
    if fd >= table_len {
        return -1;
    }
    match task.with_mut_fd_table(|table| table[fd].take()){
        Some(_) => 0,
        None => -1,
    }
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
pub fn sys_getcwd(buf: usize, len: usize) -> isize {
    let _sum_guard = SumGuard::new();
    let task = current_task().unwrap();
    task.with_cwd(|cwd| {
        let path = cwd.path();
        if len < path.len() + 1 {
            info!("[sys_getcwd]: buf len too small to recv path");
            return -1;
        } else {
            //info!("copying path: {}, len: {}", path, path.len());
            let new_buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
            new_buf.fill(0 as u8);
            let new_buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, path.len()) };
            new_buf.copy_from_slice(path.as_bytes());
            return buf as isize;
        }
    })
}

/// syscall: dup
pub fn sys_dup(old_fd: usize) -> isize {
    let task = current_task().unwrap();
    let new_fd= task.alloc_fd() as isize;
    if let Some(file) = task.with_fd_table(|table| table[old_fd].clone()) {
        task.with_mut_fd_table(|table| table[new_fd as usize] = Some(file));  
    }
    new_fd as isize
}

/// syscall: dup3
pub fn sys_dup3(old_fd: usize, new_fd: usize, _flags: u32) -> isize {
    //info!("dup3: old_fd = {}, new_fd = {}", old_fd, new_fd);
    let task = current_task().unwrap();
    let table_len = task.with_fd_table(|table|table.len());
    if old_fd >= table_len {
        return -1;
    }
    if let Some(file) = task.with_fd_table(|table| table[old_fd].clone()) {
        if new_fd < table_len {
            task.with_mut_fd_table(|table| table[new_fd] = Some(file));
        } else {
            task.with_mut_fd_table(|table| {
                table.resize(new_fd + 1, None);
                table[new_fd] = Some(file);
            });
        }
        new_fd as isize
    } else {
        -1
    }
}

/// syscall: openat
/// If the pathname given in pathname is relative, 
/// then it is interpreted relative to the directory referred to by the file descriptor dirfd 
/// (rather than relative to the current working directory of the calling process, 
/// as is done by open(2) for a relative pathname).
/// If pathname is relative and dirfd is the special value AT_FDCWD, 
/// then pathname is interpreted relative to the current working directory of the calling process (like open(2)).
/// If pathname is absolute, then dirfd is ignored.
pub fn sys_openat(dirfd: isize, pathname: *const u8, flags: u32, _mode: u32) -> isize {
    let flags = OpenFlags::from_bits(flags).unwrap();
    let task = current_task().unwrap();
    if let Some(path) = user_path_to_string(pathname) {
        if path.starts_with("/") {
            // absolute path, ignore the dirfd
            let mut ret: isize = -1;
            if let Some(file) = open_file(path.as_str(), flags) {
                task.with_mut_fd_table(|table| {
                    let fd = task.alloc_fd();
                    table[fd] = Some(file);
                    ret = fd as isize;
                });
            }
            return ret;
        } else {
            let fpath = if dirfd == AT_FDCWD {
                //info!("[sys_openat]: using current working dir");
                let cw_dentry = current_task().unwrap().with_cwd(|d|d.clone());
                rel_path_to_abs(&cw_dentry.path(), &path).unwrap()
            } else {
                // lookup in the current task's fd table
                // the inode fd points to should be a dir
                let task = current_task().unwrap();
                if let Some(dirfile) = task.with_fd_table(|table| table[dirfd as usize].clone()) {
                    let dentry = dirfile.dentry().unwrap();
                    rel_path_to_abs(&dentry.path(), &path).unwrap()
                } else {
                    info!("[sys_openat]: the dirfd not exist");
                    return -1;
                }
            };

            let mut ret: isize = -1;
            info!("fpath: {}", fpath);
            if let Some(file) = open_file(fpath.as_str(), flags) {
                let fd = task.alloc_fd();
                task.with_mut_fd_table(|table| {
                    table[fd] = Some(file);
                    ret = fd as isize;
                });
            } else {
                info!("[sys_openat]: {} not found!", fpath);
            }
            return ret;
        }
    } else {
        info!("[sys_openat]: pathname is empty!");
        return -1;
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
pub fn sys_mkdirat(dirfd: isize, pathname: *const u8, _mode: usize) -> isize {
    if let Some(path) = user_path_to_string(pathname) {
        if path.starts_with("/") {
            // absolute path, ignore the dirfd
            let dentry = global_find_dentry(&path);
            let parent = dentry.parent().unwrap();
            let parent_inode = parent.inode().unwrap();
            let name = abs_path_to_name(&path).unwrap();
            let new_inode = parent_inode.create(&name, lwext4_rust::InodeTypes::EXT4_DE_DIR).unwrap();
            dentry.set_inode(new_inode);
            dentry.set_state(dentry::DentryState::USED);
            return 0;
        } else {
            if dirfd == AT_FDCWD {
                let cw_dentry = current_task().unwrap().with_cwd(|d|d.clone());
                cw_dentry.inode().unwrap().create(&path, lwext4_rust::InodeTypes::EXT4_DE_DIR);
            } else {
                // lookup in the current task's fd table
                let task = current_task().unwrap();
                if let Some(file) = task.with_fd_table(|table| table[dirfd as usize].clone()) {
                    let inode = file.inode().unwrap();
                    inode.create(&path, lwext4_rust::InodeTypes::EXT4_DE_DIR);
                    // todo: use dentry, create a new dentry and insert iode
                } else {
                    info!("[sys_mkdirat]: the dirfd not exist");
                    return -1;
                }
            }
        }
        return 0;
    } else {
        info!("[sys_mkdirat]: pathname is empty!");
        return -1;
    }
}

/// chdir() changes the current working directory of the calling
/// process to the directory specified in path.
/// On success, zero is returned.  On error, -1 is returned, and errno
/// is set to indicate the error.
pub fn sys_chdir(path: *const u8) -> isize {
    let path = user_path_to_string(path).unwrap();
    let dentry = global_find_dentry(&path);
    if dentry.state() == DentryState::NEGATIVE {
        info!("[sys_chdir]: dentry not found");
        return -1;
    } else {
        let task = current_task().unwrap().clone();
        task.set_cwd(dentry);
        return 0;
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
pub fn sys_pipe2(pipe: *mut i32, _flags: u32) -> isize {
    let task = current_task().unwrap().clone();
    let (read_file, write_file) = make_pipe(PIPE_BUF_LEN);
    let read_fd = task.alloc_fd();
    task.with_mut_fd_table(|table| {
        table[read_fd] = Some(read_file);
    });
    let write_fd = task.alloc_fd();
    task.with_mut_fd_table(|table| {
        table[write_fd] = Some(write_file);
    });

    let _sum = SumGuard::new();
    let pipefd = unsafe { core::slice::from_raw_parts_mut(pipe, 2 * core::mem::size_of::<i32>()) };
    info!("read fd: {}, write fd: {}", read_fd, write_fd);
    pipefd[0] = read_fd as i32;
    pipefd[1] = write_fd as i32;
    0
}

/// syscall fstat
pub fn sys_fstat(fd: usize, stat_buf: usize) -> isize {
    let _sum_guard = SumGuard::new();
    let task = current_task().unwrap().clone();
    if let Some(file) = task.with_fd_table(|table| table[fd].clone()) {
        if !file.readable() {
            return -1;
        }
        let stat = file.dentry().unwrap().inode().unwrap().getattr();
        let stat_ptr = stat_buf as *mut Kstat;
        unsafe {
            *stat_ptr = stat;
        }
    } else {
        return -1;
    }
    0
}

/// syscall uname
pub fn sys_uname(uname_buf: usize) -> isize {
    let _sum_guard = SumGuard::new();
    let uname = UtsName::default();
    let uname_ptr = uname_buf as *mut UtsName;
    unsafe {
        *uname_ptr = uname;
    }
    0
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
pub fn sys_getdents64(fd: usize, buf: usize, len: usize) -> isize {
    const LEN_BEFORE_NAME: usize = 19;
    let task = current_task().unwrap().clone();
    let _sum_guard = SumGuard::new();
    let buf_slice = unsafe {
        core::slice::from_raw_parts_mut(buf as *mut u8, len)
    };
    assert!(buf_slice.len() == len);

    // get the dentry the fd points to
    if let Some(dentry) = task.with_fd_table(|table| {
        let file = table[fd].clone().unwrap();
        file.dentry()
    }) {
        let mut buf_it = buf_slice;
        let mut writen_len = 0;
        let mut pos = 0;
        for child in dentry.child_dentry() {
            assert!(child.state() != DentryState::NEGATIVE);
            // align to 8 bytes
            let c_name_len = child.name().len() + 1;
            let rec_len = (LEN_BEFORE_NAME + c_name_len + 7) & !0x7;
            let inode = child.inode().unwrap();
            let linux_dirent = LinuxDirent64 {
                d_ino: inode.inner().ino as u64,
                d_off: pos as u64,
                d_type: inode.inner().mode.bits() as u8,
                d_reclen: rec_len as u16,
            };

            //info!("[sys_getdents64] linux dirent {linux_dirent:?}");
            if writen_len + rec_len > len {
                break;
            }

            pos += 1;
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
        return writen_len as isize;
    } else {
        return -1;
    }
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
pub fn sys_unlinkat(dirfd: isize, pathname: *const u8, flags: i32) -> isize {
    let path = user_path_to_string(pathname).unwrap();
    let dentry = if path.starts_with("/") {
        global_find_dentry(&path)
    } else {
        let fpath = if dirfd == AT_FDCWD {
            //info!("[sys_openat]: using current working dir");
            let cw_dentry = current_task().unwrap().with_cwd(|d|d.clone());
            rel_path_to_abs(&cw_dentry.path(), &path).unwrap()
        } else {
            // lookup in the current task's fd table
            // the inode fd points to should be a dir
            let task = current_task().unwrap();
            if let Some(dirfile) = task.with_fd_table(|table| table[dirfd as usize].clone()) {
                let dentry = dirfile.dentry().unwrap();
                rel_path_to_abs(&dentry.path(), &path).unwrap()
            } else {
                info!("[sys_unlinkat]: the dirfd not exist");
                return -1;
            }
        };
        //info!("[sys_unlinkat]: fpath: {}", fpath);
        global_find_dentry(&fpath)
    };
    if dentry.parent().is_none() {
        warn!("cannot unlink root!");
        return -1;
    }
    let inode = dentry.inode().unwrap();
    let is_dir = inode.inner().mode == InodeMode::DIR;
    if flags == AT_REMOVEDIR && !is_dir {
        return -1;
    } else if flags != AT_REMOVEDIR && is_dir {
        return -1;
    }
    inode.unlink().expect("inode unlink failed");
    dentry.clear_inode();
    0
}
