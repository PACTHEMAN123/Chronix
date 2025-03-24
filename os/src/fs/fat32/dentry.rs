use crate::fs::{fat32::file::FatFile, vfs::{Dentry, DentryInner, DentryState, File, DCACHE}, OpenFlags, SuperBlock};
use alloc::{sync::Arc, vec::Vec};


/// fat32 file system dentry implement for VFS
pub struct FatDentry {
    inner: DentryInner,
}

unsafe impl Send for FatDentry {}
unsafe impl Sync for FatDentry {}

impl FatDentry {
    pub fn new(
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, superblock, parent)
        });
        dentry
    }
}

impl Dentry for FatDentry {
    fn inner(&self) -> &DentryInner {
        &self.inner
    }
    fn new(
        &self,
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, superblock, parent)
        });
        dentry
    }
    fn open(self: Arc<Self>, flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        let (readable, writable) = flags.read_write();
        Some(Arc::new(FatFile::new(readable, writable, self.clone())))
    }
}