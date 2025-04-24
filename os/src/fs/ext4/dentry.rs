use crate::{fs::{ext4::Ext4File, vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, File, DCACHE}, OpenFlags, SuperBlock}, syscall::SysError};

use alloc::{sync::Arc, vec::Vec};
use log::info;

use lwext4_rust::InodeTypes;

/// ext4 file system dentry implement for VFS
pub struct Ext4Dentry {
    inner: DentryInner,
}

unsafe impl Send for Ext4Dentry {}
unsafe impl Sync for Ext4Dentry {}

impl Ext4Dentry {
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

impl Dentry for Ext4Dentry {
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
    fn open(self: Arc<Self>, flags: OpenFlags) -> Option<Arc<dyn File>> {
        assert!(self.state() == DentryState::USED);
        let (readable, writable) = flags.read_write();
        Some(Arc::new(Ext4File::new(readable, writable, self.clone())))
    }
    fn load_child_dentry(self: Arc<Self>) -> Result<Vec<Arc<dyn Dentry>>, SysError> {
        //info!("in child dentry, under: {}", self.path());
        let inode = self.inode().unwrap().clone();
        let mut child_dentrys: Vec<Arc<dyn Dentry>> = Vec::new();
        // look into the children first
        // to avoid unneccsary IO
        for (_, child) in self.children().iter() {
            if child.state() == DentryState::NEGATIVE {
                // skip the invalid child dentry
                // todo: should invalid dentry stay or clean?
                continue;
            }
            child_dentrys.push(child.clone());
        }
        // try to update
        for name in inode.ls() {
            // skip the . and ..
            if name == "." || name == ".." {
                continue;
            }

            if let Some(_child_dentry) = self.get_child(&name) {
                // do nothing 
            } else {
                // not find in the mem
                // try to find by IO
                log::debug!("look up name: {}", name);
                let child_inode = inode.lookup(&name).unwrap();
                let child_dentry = self.new(
                    &name, 
                    self.superblock(), 
                    Some(self.clone()),
                );
                child_dentry.set_inode(child_inode);
                child_dentry.set_state(DentryState::USED);
                self.add_child(child_dentry.clone());
                DCACHE.lock().insert(child_dentry.path(), child_dentry.clone());
                child_dentrys.push(child_dentry);
            }
        }
        Ok(child_dentrys)
    }

    fn new_neg_dentry(self: Arc<Self>, name: &str) -> Arc<dyn Dentry> {
        let neg_dentry = Arc::new(Self {
            inner: DentryInner::new(name, self.superblock(), Some(self.clone()))
        });
        neg_dentry.set_state(DentryState::NEGATIVE);
        neg_dentry
    }
}
