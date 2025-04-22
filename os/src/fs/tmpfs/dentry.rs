use alloc::{sync::Arc, vec::Vec};

use crate::fs::{tmpfs::file::TmpFile, vfs::{Dentry, DentryInner, DentryState, File}, OpenFlags, SuperBlock};



/// tmp file system
pub struct TmpDentry {
    inner: DentryInner,
}

unsafe impl Send for TmpDentry {}
unsafe impl Sync for TmpDentry {}

impl TmpDentry {
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

impl Dentry for TmpDentry {
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
    fn open(self: Arc<Self>, _flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        Some(Arc::new(TmpFile::new(self.clone())))
    }
    fn load_child_dentry(self: Arc<Self>) -> Vec<Arc<dyn Dentry>> {
        let mut child_dentrys: Vec<Arc<dyn Dentry>> = Vec::new();
        for (_, child) in self.children().iter() {
            if child.state() == DentryState::NEGATIVE {
                continue;
            }
            child_dentrys.push(child.clone());
        }
        child_dentrys
    }
    fn new_neg_dentry(self: Arc<Self>, name: &str) -> Arc<dyn Dentry> {
        let neg_dentry = Arc::new(Self {
            inner: DentryInner::new(name, self.superblock(), Some(self.clone()))
        });
        neg_dentry.set_state(DentryState::NEGATIVE);
        neg_dentry
    }
}