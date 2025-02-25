//! file system module: offer the file system interface
//! define the file trait
//! impl File for OSInode in `inode.rs`
//! impl Stdin and Stdout in `stdio.rs`
pub mod inode;
pub mod stdio;
pub mod disk;
pub mod ext4fs;

use crate::mm::UserBuffer;
/// File trait
pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use inode::{list_apps, open_file, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};
pub use disk::Disk;
pub use ext4fs::{Ext4FileSystem, Inode};
