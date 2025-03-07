//! VFS
//! Chronix Virtual File System
//! all file system should implement the VFS trait to plugin Chronix

mod superblock;
mod inode;

pub use superblock::{SuperBlockInner, SuperBlock};
pub use inode::{InodeInner, Inode};
