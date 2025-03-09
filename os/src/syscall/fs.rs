//! File and filesystem-related syscalls
use log::info;

use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: usize, len: usize) -> isize {
    let token = current_user_token();
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

pub fn sys_read(fd: usize, buf: usize, len: usize) -> isize {
    //info!("in sys_read");
    let token = current_user_token();
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

pub async fn sys_open(path: usize, flags: u32) -> isize {
    //info!("in sys_open");
    let task = current_task().unwrap();
    let token = current_user_token();
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

pub fn sys_dup(old_fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut new_fd: isize = -1;
    if let Some(file) = task.with_fd_table(|table| table[old_fd].clone()) {
        new_fd = task.alloc_fd() as isize;
        task.with_mut_fd_table(|table| table[new_fd as usize] = Some(file));  
    }
    new_fd as isize
}

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
