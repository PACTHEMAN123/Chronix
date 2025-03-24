//! VFS for lwext4_rust
extern crate lwext4_rust;

pub use lwext4_rust::InodeTypes;

mod disk;
mod inode;
mod file;
mod superblock;
mod dentry;
mod fstype;

pub use disk::Disk;
use hal::println;
pub use inode::Ext4Inode;
pub use file::Ext4File;
pub use superblock::Ext4SuperBlock;
pub use dentry::Ext4Dentry;
pub use fstype::Ext4FSType;
use virtio_drivers::PAGE_SIZE;

use crate::fs::vfs::inode::InodeMode;

use super::vfs::DCACHE;

#[allow(unused)]
pub fn page_cache_test() {
    // create a new inode at root
    let root_dentry = DCACHE.lock().get("/").unwrap().clone();
    let root = root_dentry.inode().unwrap();
    let inode = root.create("/page_cache_test.txt", InodeMode::FILE).unwrap();

    // write something in inode using cache mode
    let write_buf = [0xAAu8; 4 * PAGE_SIZE];
    let write_size = inode.clone().cache_write_at(0, &write_buf).expect("cache write failed");

    // read something 
    let mut read_buf = [0u8; 4 * PAGE_SIZE];
    let read_size = inode.cache_read_at(0, &mut read_buf).expect("cache read failed");
    assert!(read_size == write_size);
    assert_eq!(write_buf, read_buf, "data not match!");

    // drop the inode
    // drop(inode);

    // get the same inode and read again, make sure the data has been write back
    let inode = root.create("/page_cache_test.txt", InodeMode::FILE).unwrap();
    let mut read_buf_2 = [0u8; 4 * PAGE_SIZE];
    let read_size_2 = inode.clone().cache_read_at(0, &mut read_buf_2).expect("cache read failed at second time");
    assert!(read_size == read_size_2);
    assert_eq!(read_buf, read_buf_2, "data not match after flush");

    // remove the inode in fs
    inode.unlink();

    println!("page cache test passed!");
}