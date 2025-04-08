use alloc::sync::Arc;

use crate::fs::{vfs::{Dentry, DentryInner, File}, OpenFlags, SuperBlock};

use super::tty::TtyFile;

/// the /dev dir
pub struct DevDentry {
    inner: DentryInner,
}

unsafe impl Send for DevDentry {}
unsafe impl Sync for DevDentry {}

impl DevDentry {
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

impl Dentry for DevDentry {
    fn inner(&self) -> &DentryInner {
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
        // ugly way of opening the file
        let name = self.name();
        match name {
            "tty" => Some(TtyFile::new(self.clone())),
            _ => {
                panic!()
            }
        }
    }
}