//! VFS for rust fatfs

pub mod fstype;
pub mod disk;
pub mod inode;
pub mod dentry;
pub mod file;
pub mod superblock;

use fatfs::Error;

pub use sys_error::SysError;

use crate::syscall::sys_error;

/// match fat32 error to sys error
/// (todo): match more error
pub fn as_vfs_err(err: Error<()>) -> SysError {
    match err {
        Error::AlreadyExists => SysError::EEXIST,
        Error::CorruptedFileSystem => SysError::EIO,
        Error::DirectoryIsNotEmpty => SysError::ENOTEMPTY,
        Error::InvalidInput
        | Error::InvalidFileNameLength
        | Error::UnsupportedFileNameCharacter => SysError::EIO,
        Error::NotEnoughSpace => SysError::ENOMEM,
        Error::NotFound => SysError::ENOENT,
        Error::UnexpectedEof => SysError::EIO,
        Error::WriteZero => SysError::EIO,
        Error::Io(_) => SysError::EIO,
        _ => SysError::EIO,

    }
}