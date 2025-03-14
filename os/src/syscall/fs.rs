//! File and filesystem-related syscalls
use alloc::string::ToString;
use log::info;

use crate::fs::{
    ext4::open_file,
    vfs::{File, dentry::global_find_dentry, dentry},
    OpenFlags,
    AT_FDCWD,
};
use crate::utils::{
    path::*,
    string::*,
};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer, UserCheck};
use crate::processor::processor::{current_processor,current_task,current_user_token};

/// syscall: write
pub fn sys_write(fd: usize, buf: usize, len: usize) -> isize {
    let token = current_user_token(&current_processor());
    let task = current_task().unwrap();
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
        file.write(UserBuffer::new(translated_byte_buffer(token, buf as *const u8, len))) as isize
    } else {
        -1
    }
}


/// syscall: read
pub fn sys_read(fd: usize, buf: usize, len: usize) -> isize {
    //info!("in sys_read");
    let token = current_user_token(&current_processor());
    let task = current_task().unwrap();
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
        file.read(UserBuffer::new(translated_byte_buffer(token, buf as *const u8, len))) as isize
    } else {
        -1
    }
}

/// syscall: open
pub async fn sys_open(path: usize, flags: u32) -> isize {
    //info!("in sys_open");
    let task = current_task().unwrap();
    let token = current_user_token(&current_processor());
    let path = translated_str(token, path as *const u8);
    let mut ret = -1;
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        task.with_mut_fd_table(|table| {
            let fd = task.alloc_fd();
            table[fd] = Some(inode);
            ret = fd as isize;
        });
    }
    ret
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
    let user_check = UserCheck::new();
    //info!("[sys_getcwd]: buf addr: {:x}, size: {}", buf, len);
    user_check.check_write_slice(buf as *mut u8, len);
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
    let mut new_fd: isize = -1;
    if let Some(file) = task.with_fd_table(|table| table[old_fd].clone()) {
        new_fd = task.alloc_fd() as isize;
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
            if dirfd == AT_FDCWD {
                //info!("[sys_openat]: using current working dir");
                let cw_dentry = current_task().unwrap().with_cwd(|d|d.clone());
                let fpath = cw_dentry.path() + &path;
                //info!("[sys_openat]: full path: {}", fpath);
                let mut ret: isize = -1;
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
            } else {
                // lookup in the current task's fd table
                let task = current_task().unwrap();
                if let Some(_file) = task.with_fd_table(|table| table[dirfd as usize].clone()) {
                    info!("[sys_openat]: not support");
                    // todo: replace inode to dentry in File object
                    return -1;
                } else {
                    info!("[sys_openat]: the dirfd not exist");
                    return -1;
                }
            }
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
