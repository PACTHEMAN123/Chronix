# overview

This file is written to demonstrate the relationship of the files in this directory and layer from bottom to top.

# layers

## below the lwext4 filesystem

### design idea

in the lwext4 filesystem source code, the `KernelDevOp` trait is defined in `blockdev.rs`.

```rust
// in lwext4_rust/src/blockdev.rs
pub trait KernelDevOp {
    //type DevType: ForeignOwnable + Sized + Send + Sync = ();
    type DevType;

    //fn write(dev: <Self::DevType as ForeignOwnable>::Borrowed<'_>, buf: &[u8]) -> Result<usize, i32>;
    fn write(dev: &mut Self::DevType, buf: &[u8]) -> Result<usize, i32>;
    fn read(dev: &mut Self::DevType, buf: &mut [u8]) -> Result<usize, i32>;
    fn seek(dev: &mut Self::DevType, off: i64, whence: i32) -> Result<i64, i32>;
    fn flush(dev: &mut Self::DevType) -> Result<usize, i32>
    where
        Self: Sized;
}
```

this trait is the basic of the filesystem, since it needs to operate on the device block.

### implement

Firstly, we need to implement the Hal trait for the block device using the physical memory allocation in Chronix. (Necessary for the `VirtIOBlk` struct.)

Secondly, we need to have a disk. In `disk.rs`, we initialize the `Disk` struct with a `VirtIOBlk` struct and implement some basic operations for it.

Thirdly, we need to implement the `KernelDevOp` trait for the `Disk` struct. After that, we can use the `Disk` struct as a device for the filesystem.

(We basically use the code in examples/src.)  

## lwext4 filesystem

see the source code of the lwext4 filesystem. It provides the interface of the filesystem: `Ext4File` 

>Wrap up the file operation struct Ext4File. It can provide the file system interfaces for the upper layer of Rust OS. `Ext4File` operations include: `file_read`, `file_write`, `file_seek`, `file_open`, `file_close`, `file_rename`, `lwext4_dir_entries` ...

## above the lwext4 filesystem

the relationship:
- File
    - Stdio
    - OSInode
        - FileWrapper + VfsNodeOps
            - Ext4File
        - Ext4FileSystem + VfsOps

for the implement of `VfsOps` for `Ext4FileSystem` and `VfsNodeOps` for `FileWrapper`, see the source code in `ext4fs.rs`.

we then use the FileWrapper and Ext4FileSystem to implement the OSInode.

as for the rest of the code (about file system), we simply use the code in rCore ch6.






