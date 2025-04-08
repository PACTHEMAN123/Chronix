use alloc::{collections::btree_map::BTreeMap, string::String, sync::Arc};

use crate::{fs::{vfs::{inode::InodeMode, Inode, InodeInner}, SuperBlock}, sync::mutex::SpinNoIrqLock};

use super::tty::TtyInode;

/// dev fs inode
/// notice that can only be dir
/// since the not dir inode must be device
pub struct DevInode {
    inner: InodeInner,
    // /dev dir inode will use the map to do the lookups
    childs: SpinNoIrqLock<BTreeMap<String, Arc<dyn Inode>>>,
}

impl DevInode {
    pub fn new(super_block: Arc<dyn SuperBlock>) -> Arc<Self> {
        let inner = InodeInner::new(super_block, InodeMode::DIR, 0);
        let childs = SpinNoIrqLock::new(BTreeMap::new());
        Arc::new(Self { inner, childs })
    }
}

impl Inode for DevInode {
    fn inner(&self) -> &InodeInner {
        &self.inner
    }

    fn lookup(&self, name: &str) -> Option<Arc<dyn Inode>> {
        let childs = self.childs.lock();
        let inode = childs.get(name).unwrap().clone();
        Some(inode)
    }
}