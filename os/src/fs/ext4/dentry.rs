use crate::fs::{vfs::{inode::InodeMode, Dentry, DentryInner, DentryState, DCACHE}, SuperBlock};

use alloc::{sync::Arc, vec::Vec};
use log::info;

use lwext4_rust::InodeTypes;

/// ext4 file system dentry implement for VFS
pub struct Ext4Dentry {
    inner: DentryInner,
}

unsafe impl Send for Ext4Dentry {}
unsafe impl Sync for Ext4Dentry {}

impl Dentry for Ext4Dentry {
    fn inner(&self) -> &DentryInner {
        &self.inner
    }

    fn walk(self: Arc<Self>, path: &str) -> Arc<dyn Dentry> {
        // get current inode
        let mut current_inode = self.inode().unwrap();
        let mut current_dentry = self.clone();
        // break down the path: string a/b/c -> vec [a, b, c]
        let name_vec: Vec<&str> = path
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        // use the vec to walk, loop
        // if the element exist, keeping walking
        // if not exist, stop.
        for (_idx, name) in name_vec.iter().enumerate() {
            if let Some(child) = current_inode.lookup(name) {   
                // on the path, insert into dcache
                // construct along the way
                let path_dentry = Ext4Dentry::new(
                    name,
                    self.superblock(),
                    Some(current_dentry)
                );
                path_dentry.set_inode(child.clone());
                path_dentry.set_state(DentryState::USED);
                let key = path_dentry.path();
                // info!("[DCACHE]: insert key: {}", key);
                // (todo): insert op may be duplicate
                DCACHE.lock().insert(key, path_dentry.clone());
                current_dentry = path_dentry;
                current_inode = child;
            } else {
                // not found, construct a negative dentry
                let neg_dentry = Ext4Dentry::new(
                    name,
                    self.superblock(),
                    Some(current_dentry)
                );
                neg_dentry.set_inode(current_inode);
                neg_dentry.set_state(DentryState::NEGATIVE);
                //info!("[DCACHE]: insert key: {}", neg_dentry.path());
                DCACHE.lock().insert(neg_dentry.path(), neg_dentry.clone());
                return neg_dentry;
            }
        }
        return current_dentry.clone();
    }

    fn child_dentry(self: Arc<Self>) -> Vec<Arc<dyn Dentry>> {
        info!("in child dentry, under: {}", self.path());
        let inode = self.inode().unwrap().clone();
        let inner = inode.inner();
        assert!(inner.mode == InodeMode::from_inode_type(InodeTypes::EXT4_DE_DIR));
        let mut child_dentrys: Vec<Arc<dyn Dentry>> = Vec::new();
        for name in inode.ls() {
            let child_inode = inode.lookup(&name).unwrap();
            
            let child_dentry = Ext4Dentry::new(
                &name, 
                self.superblock(), 
                Some(self.clone()),
            );
            child_dentry.set_inode(child_inode);
            child_dentry.set_state(DentryState::USED);
            DCACHE.lock().insert(child_dentry.path(), child_dentry.clone());
            child_dentrys.push(child_dentry);
        }
        child_dentrys
    }

}

impl Ext4Dentry {
    /// create a new Ext4 dentry
    pub fn new(
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<Self> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, superblock, parent)
        });
        dentry
    }
}