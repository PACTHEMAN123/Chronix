//! file system support for Task

use alloc::{sync::Arc, vec::Vec};
use fatfs::info;
use log::warn;

use crate::{fs::{devfs::tty::TTY, vfs::{Dentry, File}, OpenFlags, Stdin}, syscall::{misc::RLimit, SysError}};

use super::task::TaskControlBlock;

#[derive(Clone)]
/// the fd table
pub struct FdTable {
    /// the inner table
    pub fd_table: Vec<Option<FdInfo>>,
    /// resource limit: max fds
    pub rlimit: RLimit,
}

/// Max file descriptors counts
pub const MAX_FDS: usize = 1024;

impl FdTable {
    /// new and init fd table
    pub fn new() -> Self {
        let mut table: Vec<Option<FdInfo>> = Vec::new();
        // 0 -> stdin (todo: use TTY instead of Stdin, now TTY is not support for loop reading)
        //let stdin = Arc::new(Stdin);
        let tty_file = TTY.get().unwrap().clone();
        let stdin = tty_file.clone();
        //stdin.set_flags(OpenFlags::empty());
        // 1 -> stdout
        let stdout = tty_file.clone();
        //stdout.set_flags(OpenFlags::O_WRONLY);
        // 2 -> stderr
        let stderr = tty_file.clone();
        //stderr.set_flags(OpenFlags::O_WRONLY);
        table.push(Some(FdInfo { file: stdin, flags: FdFlags::empty() }));
        table.push(Some(FdInfo { file: stdout, flags: FdFlags::empty() }));
        table.push(Some(FdInfo { file: stderr, flags: FdFlags::empty() }));
        
        Self { 
            fd_table: table,
            rlimit: RLimit { rlim_cur: MAX_FDS, rlim_max: MAX_FDS }
        }
    }
    /// allocate a new fd for the task
    /// will not expend the fd table
    pub fn alloc_fd(&mut self) -> Result<usize, SysError> {
        if let Some (fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            Ok(fd)
        } else if self.fd_table.len() < self.rlimit.rlim_max {
            self.fd_table.push(None);
            Ok(self.fd_table.len() - 1)
        } else {
            Err(SysError::EBADF)
        }
    }
    /// allocate a new fd greater or equal to given bound
    /// expend the table if the max fd is not enough
    pub fn alloc_fd_from(&mut self, bound: usize) -> Result<usize, SysError> {
        if bound > self.rlimit.rlim_max {
            return Err(SysError::EBADF)
        }

        if self.fd_table.len() <= bound {
            // expand the fd table
            self.fd_table.resize(bound + 1, None);
        }
        if let Some(fd) = (bound..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            Ok(fd)
        } else if self.fd_table.len() < self.rlimit.rlim_max {
            // no space, append to end
            self.fd_table.push(None);
            Ok(self.fd_table.len() - 1)
        } else {
            return Err(SysError::EMFILE)
        }
    }
    /// get the fd_info using fd
    pub fn get_fd_info(&self, fd: usize) -> Result<FdInfo, SysError> {
        if fd >= self.fd_table.len() {
            log::warn!("[get_fd_info_1] fd {} is not valid",fd);
            return Err(SysError::EBADF);
        }
        if let Some(fdinfo) = self.fd_table[fd].clone() {
            return Ok(fdinfo)
        } else {
            log::warn!("[get_fd_info_2] fd {} is not valid",fd);
            return Err(SysError::EBADF);
        }
    }
    /// get the mut fd_info using fd
    pub fn get_mut_fd_info(&mut self, fd: usize) -> Result<&mut FdInfo, SysError> {
        if fd >= self.fd_table.len() {
            log::warn!("[get_mut_fd_info_1] fd {} is not valid",fd);
            return Err(SysError::EBADF);
        }
        self.fd_table[fd].as_mut().ok_or_else(||{
            log::warn!("[get_mut_fd_info_2] fd {} is not valid",fd);
            SysError::EBADF
        })
    }
    /// get the file using fd
    /// error if not found
    pub fn get_file(&self, fd: usize) -> Result<Arc<dyn File>, SysError> {
        if fd >= self.fd_table.len() {
            log::warn!("[get_file] fd {} is not valid",fd);
            return Err(SysError::EBADF);
        }
        if let Some(fdinfo) = self.fd_table[fd].clone() {
            return Ok(fdinfo.file)
        } else {
            log::warn!("[get_file_2] fd {} is not valid, table len {}, is true: {}",fd,self.fd_table.len(), !self.fd_table[fd].is_none());
            return Err(SysError::EBADF);
        }
    }
    /// put the file into given fd slot
    pub fn put_file(&mut self, fd: usize, fd_info: FdInfo) -> Result<(), SysError> {
        if fd >= self.fd_table.len() {
            log::warn!("[put_file] fd {} is not valid",fd);
            return Err(SysError::EBADF);
        }
        self.fd_table[fd] = Some(fd_info);
        Ok(())
    }
    /// clear the slot using given fd
    pub fn remove(&mut self, fd: usize) -> Result<(), SysError> {
        if fd >= self.fd_table.len() {
            log::warn!("[fs::remove_1] fd {} is not valid",fd);
            return Err(SysError::EBADF);
        } else if self.fd_table[fd].is_none() {
            log::warn!("[fs::remove_2] fd {} is not valid",fd);
            return Err(SysError::EBADF);
        } else {
            self.fd_table[fd] = None;
            Ok(())
        } 
    }
    /// dup fd in file table with bound, return new fd
    /// new fd will use the given flags
    pub fn dup_with_bound(&mut self, old_fd: usize, bound: usize, flags: FdFlags) -> Result<usize, SysError> {
        log::debug!("dup with bound: old fd {}, bound {}", old_fd, bound);
        let file = self.get_file(old_fd)?;
        let fd_info = FdInfo {file, flags};
        let new_fd = self.alloc_fd_from(bound)?;
        self.put_file(new_fd, fd_info)?;
        assert!(new_fd >= bound);
        Ok(new_fd)
    }
    /// dup fd
    /// new fd will have empty flags
    /// no bound
    pub fn dup_no_flag(&mut self, old_fd: usize) -> Result<usize, SysError> {
        let file = self.get_file(old_fd)?;
        let fd_info = FdInfo {file, flags: FdFlags::empty()};
        let new_fd = self.alloc_fd_from(0)?;
        self.put_file(new_fd, fd_info)?;
        Ok(new_fd)
    }
    /// call by dup3
    /// new fd will use the given flags
    pub fn dup3(&mut self, old_fd: usize, new_fd: usize, flags: FdFlags) -> Result<usize, SysError> {
        let file = self.get_file(old_fd)?;
        if self.fd_table.len() <= new_fd {
            self.fd_table.resize(new_fd.checked_add(1).ok_or(SysError::EBADF)?, None);
        }
        self.fd_table[new_fd] = Some(FdInfo {file, flags});
        Ok(new_fd)
    }
    /// call by dup3
    /// new fd will use the old fd's flag
    pub fn dup3_with_flags(&mut self, old_fd: usize, new_fd: usize) -> Result<usize, SysError> {
        let fd_info = self.get_fd_info(old_fd)?;
        if self.fd_table.len() <= new_fd {
            self.fd_table.resize(new_fd.checked_add(1).ok_or(SysError::EBADF)?, None);
        }
        self.fd_table[new_fd] = Some(fd_info);
        Ok(new_fd)
    }
    /// get rlimit
    pub fn rlimit(&self) -> RLimit {
        self.rlimit
    }
    /// set rlimit
    pub fn set_rlimit(&mut self, rlimit: RLimit) {
        self.rlimit = rlimit;
        if rlimit.rlim_max <= self.fd_table.len() {
            panic!("not finish");
            // self.fd_table.truncate(self.rlimit.rlim_max)
        }
    }
    /// handle close-on-exec flag
    pub fn do_close_on_exec(&mut self) {
        for fd_info in self.fd_table.iter_mut() {
            if let Some(fd) = fd_info {
                if fd.flags.contains(FdFlags::CLOEXEC){
                    *fd_info = None;
                }
            }
        }
    }
}


#[derive(Clone)]
/// fd info
pub struct FdInfo {
    /// the file it points to
    pub file: Arc<dyn File>,
    /// fd flags
    pub flags: FdFlags,
}

impl FdInfo {
    ///
    pub fn flags(&self) -> FdFlags {
        self.flags
    }
    ///
    pub fn set_flags(&mut self, flags: FdFlags) {
        self.flags = flags
    }
}

bitflags::bitflags! {
    /// Defined in <bits/fcntl-linux.h>.
    pub struct FdFlags: u8 {
        ///
        const CLOEXEC = 1;
    }
}

impl From<OpenFlags> for FdFlags {
    fn from(value: OpenFlags) -> Self {
        if value.contains(OpenFlags::O_CLOEXEC) {
            FdFlags::CLOEXEC
        } else {
            FdFlags::empty()
        }
    }
}


/// for file system
impl TaskControlBlock {
    /// get the current working dir
    pub fn cwd(&self) -> Arc<dyn Dentry> {
        self.cwd.lock().clone()
    } 
    /// change the current working dir
    pub fn set_cwd(&self, dentry: Arc<dyn Dentry>) {
        log::info!("switching task {}'s cwd to {}", self.gettid(), dentry.path());
        *self.cwd.lock() = dentry;
    }
    
    
}