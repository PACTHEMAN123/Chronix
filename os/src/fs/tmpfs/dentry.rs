use alloc::{sync::Arc, vec::Vec};

use crate::{fs::{tmpfs::{file::TmpFile, inode::TmpInode}, vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, File}, OpenFlags, SuperBlock}, syscall::SysError};



/// tmp file system
pub struct TmpDentry {
    inner: DentryInner,
}

unsafe impl Send for TmpDentry {}
unsafe impl Sync for TmpDentry {}

impl TmpDentry {
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

impl Dentry for TmpDentry {
    fn dentry_inner(&self) -> &DentryInner {
        &self.inner
    }
    fn new(&self,
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, parent)
        });
        dentry
    }

    /// tmpfs should support O_TMPFILE open flags
    /// Create an unnamed temporary regular file.  The pathname
    /// argument specifies a directory; an unnamed inode will be
    /// created in that directory's filesystem.  Anything written
    /// to the resulting file will be lost when the last file
    /// descriptor is closed, unless the file is given a name.
    fn open(self: Arc<Self>, flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        if flags.contains(OpenFlags::O_TMPFILE) {
            // only the fd table will hold the file
            let sb = self.inode().unwrap().inode_inner().super_block.clone().unwrap();
            let new_inode = TmpInode::new(sb, InodeMode::FILE);
            let new_dentry = TmpDentry::new("unname,shit!", None);
            new_dentry.set_inode(new_inode);
            new_dentry.set_state(DentryState::NEGATIVE); // cannot use dir to find it
            return Some(Arc::new(TmpFile::new(new_dentry)));
        }

        Some(Arc::new(TmpFile::new(self.clone())))
    }

    fn load_child_dentry(self: Arc<Self>) -> Result<Vec<Arc<dyn Dentry>>, SysError> {
        let mut child_dentrys: Vec<Arc<dyn Dentry>> = Vec::new();
        for (_, child) in self.children().iter() {
            if child.state() == DentryState::NEGATIVE {
                continue;
            }
            child_dentrys.push(child.clone());
        }
        Ok(child_dentrys)
    }
    fn new_neg_dentry(self: Arc<Self>, name: &str) -> Arc<dyn Dentry> {
        let neg_dentry = Arc::new(Self {
            inner: DentryInner::new(name, Some(self.clone()))
        });
        neg_dentry.set_state(DentryState::NEGATIVE);
        neg_dentry
    }
    fn clear_inode(&self) {
        // like tmpfile(), its ok to read / write the file
        // even it is unlink, as long as someone owns the file (like fd table)
        // since in tmpfs, dentry is the only thing that holds inode
        // should not drop inode here.
        self.set_state(DentryState::NEGATIVE);
    }
}