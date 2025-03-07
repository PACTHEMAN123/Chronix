//! VFS for lwext4_rust

mod disk;
mod inode;
mod file;
mod superblock;

pub use disk::Disk;
pub use inode::Ext4Inode;
pub use file::{list_apps, open_file, OSInode, OpenFlags};
pub use superblock::Ext4SuperBlock;