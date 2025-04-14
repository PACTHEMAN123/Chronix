use alloc::{sync::Arc, vec::Vec};

use crate::fs::{simplefs::file::SpFile, vfs::{Dentry, DentryInner, DentryState, File}, OpenFlags, SuperBlock};


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
    fn load_child_dentry(self: Arc<Self>) -> Vec<Arc<dyn Dentry>> {
        let mut child_dentrys = Vec::new();
        let children = self.children();
        for (_, child) in children.iter() {
            child_dentrys.push(child.clone());
        }
        child_dentrys
    }
    fn open(self: Arc<Self>, _flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        Some(SpFile::new(self.clone()))
    }
}