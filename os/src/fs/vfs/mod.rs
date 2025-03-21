//! VFS
//! Chronix Virtual File System
//! all file system should implement the VFS trait to plugin Chronix

pub mod superblock;
pub mod inode;
pub mod file;
pub mod dentry;
pub mod fstype;

pub use superblock::{SuperBlockInner, SuperBlock};
pub use inode::{InodeInner, Inode};
pub use file::{FileInner, File};
pub use dentry::{DentryInner, Dentry, DCACHE, DentryState};
