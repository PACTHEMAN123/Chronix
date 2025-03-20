//! vfs file system type

use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};

use crate::{devices::BlockDevice, sync::mutex::SpinNoIrqLock};

use super::{Dentry, SuperBlock};

pub struct FSTypeInner {
    /// name of the file system type
    name: String,
    /// the super blocks
    pub supers: SpinNoIrqLock<BTreeMap<String, Arc<dyn SuperBlock>>>,
}

impl FSTypeInner {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            supers: SpinNoIrqLock::new(BTreeMap::new()),
        }
    }
}

pub trait FSType: Send + Sync {
    /// get the base fs type
    fn inner(&self) -> &FSTypeInner;
    /// mount a new instance of this file system
    fn mount(self: Arc<Self>, name: &str, parent: Option<Arc<dyn Dentry>>, flags: MountFlags, dev: Option<Arc<dyn BlockDevice>>) -> Option<Arc<dyn Dentry>>;
    /// shutdown a instance of this file system
    fn kill_sb(&self) -> isize;
    /// get the file system name
    fn name(&self) -> &str {
        &self.inner().name
    }
    /// use the mount path to get the super block
    fn get_sb(&self, abs_mount_path: &str) -> Option<Arc<dyn SuperBlock>> {
        self.inner()
        .supers
        .lock()
        .get(abs_mount_path)
        .cloned()
    }
    /// add a new super block
    fn add_sb(&self, abs_mount_path: &str, super_block: Arc<dyn SuperBlock>) {
        self.inner()
        .supers
        .lock()
        .insert(abs_mount_path.to_string(), super_block);
    }
}

bitflags::bitflags! {
    pub struct FileSystemFlags:u32{
        /// The file system requires a device.
        const REQUIRES_DEV = 0x1;
        /// The options provided when mounting are in binary form.
        const BINARY_MOUNTDATA = 0x2;
        /// The file system has a subtype. It is extracted from the name and passed in as a parameter.
        const HAS_SUBTYPE = 0x4;
        /// The file system can be mounted by userns root.
        const USERNS_MOUNT = 0x8;
        /// Disables fanotify permission events.
        const DISALLOW_NOTIFY_PERM = 0x10;
        /// The file system has been updated to handle vfs idmappings.
        const ALLOW_IDMAP = 0x20;
        /// FS uses multigrain timestamps
        const MGTIME = 0x40;
        /// The file systen will handle `d_move` during `rename` internally.
        const RENAME_DOES_D_MOVE = 0x8000;
    }
}

bitflags::bitflags! {
    pub struct MountFlags:u32 {
        /// This filesystem is mounted read-only.
        const MS_RDONLY = 1;
        /// The set-user-ID and set-group-ID bits are ignored by exec(3) for executable files on this filesystem.
        const MS_NOSUID = 1 << 1;
        /// Disallow access to device special files on this filesystem.
        const MS_NODEV = 1 << 2;
        /// Execution of programs is disallowed on this filesystem.
        const MS_NOEXEC = 1 << 3;
        /// Writes are synched to the filesystem immediately (see the description of O_SYNC in open(2)).
        const MS_SYNCHRONOUS = 1 << 4;
        /// Alter flags of a mounted FS
        const MS_REMOUNT = 1 << 5;
        /// Allow mandatory locks on an FS
        const MS_MANDLOCK = 1 << 6;
        /// Directory modifications are synchronous
        const MS_DIRSYNC = 1 << 7;
        /// Do not follow symlinks
        const MS_NOSYMFOLLOW = 1 << 8;
        /// Do not update access times.
        const MS_NOATIME = 1 << 10;
        /// Do not update directory access times
        const MS_NODEIRATIME = 1 << 11;
        const MS_BIND = 1 << 12;
        const MS_MOVE = 1 << 13;
        const MS_REC = 1 << 14;
        /// War is peace. Verbosity is silence.
        const MS_SILENT = 1 << 15;
        /// VFS does not apply the umask
        const MS_POSIXACL = 1 << 16;
        /// change to unbindable
        const MS_UNBINDABLE = 1 << 17;
        /// change to private
        const MS_PRIVATE = 1 << 18;
        /// change to slave
        const MS_SLAVE = 1 << 19;
        /// change to shared
        const MS_SHARED = 1 << 20;
        /// Update atime relative to mtime/ctime.
        const MS_RELATIME = 1 << 21;
        /// this is a kern_mount call
        const MS_KERNMOUNT = 1 << 22;
        /// Update inode I_version field
        const MS_I_VERSION = 1 << 23;
        /// Always perform atime updates
        const MS_STRICTATIME = 1 << 24;
        /// Update the on-disk [acm]times lazily
        const MS_LAZYTIME = 1 << 25;
        /// These sb flags are internal to the kernel
        const MS_SUBMOUNT = 1 << 26;
        const MS_NOREMOTELOCK = 1 << 27;
        const MS_NOSEC = 1 << 28;
        const MS_BORN = 1 << 29;
        const MS_ACTIVE = 1 << 30;
        const MS_NOUSER = 1 << 31;
    }
}