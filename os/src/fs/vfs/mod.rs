//! VFS
//! Chronix Virtual File System
//! all file system should implement the VFS trait to plugin Chronix

mod superblock;

pub use superblock::{SuperBlockInner, SuperBlock};