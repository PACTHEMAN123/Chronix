use crate::{fs::{fat32::file::FatFile, vfs::{Dentry, DentryInner, DentryState, File, DCACHE}, OpenFlags, SuperBlock}, syscall::SysError};
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
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, parent)
        });
        dentry
    }
}

impl Dentry for FatDentry {
    fn dentry_inner(&self) -> &DentryInner {
        &self.inner
    }
    fn new(
        &self,
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, parent)
        });
        dentry
    }
    fn open(self: Arc<Self>, flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        let (readable, writable) = flags.read_write();
        Some(Arc::new(FatFile::new(readable, writable, self.clone())))
    }
    fn new_neg_dentry(self: Arc<Self>, name: &str) -> Result<Arc<dyn Dentry>, SysError> {
        let neg_dentry = Arc::new(Self {
            inner: DentryInner::new(name, Some(self.clone()))
        });
        neg_dentry.set_state(DentryState::NEGATIVE);
        Ok(neg_dentry)
    }
}