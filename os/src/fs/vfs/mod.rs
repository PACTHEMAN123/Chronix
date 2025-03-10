//! VFS
//! Chronix Virtual File System
//! all file system should implement the VFS trait to plugin Chronix

mod superblock;
mod inode;
mod file;
mod dentry;

pub use superblock::{SuperBlockInner, SuperBlock};
pub use inode::{InodeInner, Inode};
pub use file::{FileInner, File};
pub use dentry::{DentryInner, Dentry, DCACHE, DentryState};
