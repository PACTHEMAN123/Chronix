use alloc::{sync::Arc, vec::Vec};

use crate::{fs::{simplefs::file::SpFile, vfs::{Dentry, DentryInner, DentryState, File}, OpenFlags, SuperBlock}, syscall::SysError};


pub struct SpDentry {
    inner: DentryInner,
}

unsafe impl Send for SpDentry {}
unsafe impl Sync for SpDentry {}

impl SpDentry {
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

impl Dentry for SpDentry {
    fn dentry_inner(&self) -> &DentryInner {
        &self.inner
    }
    fn new(&self,
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, superblock, parent)
        });
        dentry
    }
    fn load_child_dentry(self: Arc<Self>) -> Result<Vec<Arc<dyn Dentry>>, SysError> {
        let mut child_dentrys = Vec::new();
        let children = self.children();
        for (_, child) in children.iter() {
            child_dentrys.push(child.clone());
        }
        Ok(child_dentrys)
    }
    fn open(self: Arc<Self>, _flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        Some(SpFile::new(self.clone()))
    }
    fn new_neg_dentry(self: Arc<Self>, name: &str) -> Arc<dyn Dentry> {
        let neg_dentry = Arc::new(Self {
            inner: DentryInner::new(name, self.superblock(), Some(self.clone()))
        });
        neg_dentry.set_state(DentryState::NEGATIVE);
        neg_dentry
    }
}