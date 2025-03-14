//! virtual file system dentry

use core::default;

use crate::{fs::vfs::dentry, sync::mutex::SpinNoIrqLock};

use super::{superblock, Inode, SuperBlock};

use alloc::{
    collections::btree_map::BTreeMap, string::{String, ToString}, sync::{Arc, Weak}
};
use log::{info, warn};


/// basic dentry object
pub struct DentryInner {
    /// name of the inode it points to
    pub name: String,
    /// inode it points to
    pub inode: SpinNoIrqLock<Option<Arc<dyn Inode>>>,
    /// superblock of the inode belongs to
    pub superblock: Weak<dyn SuperBlock>,
    /// parent
    pub parent: Option<Weak<dyn Dentry>>,
    /// state
    pub state: SpinNoIrqLock<DentryState>,
}

impl DentryInner {
    /// create a unused dentry: no children in it
    pub fn new(
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Self {
        let superblock = Arc::downgrade(&superblock);
        let inode = SpinNoIrqLock::new(None);
        Self {
            name: name.to_string(),
            superblock,
            inode,
            parent: parent.map(|p| Arc::downgrade(&p)),
            state: SpinNoIrqLock::new(DentryState::UNUSED),
        }
    }
}

/// dentry method that all fs need to implement
pub trait Dentry: Send + Sync {
    /// get the inner dentry
    fn inner(&self) -> &DentryInner;
    /// get the inode it points to
    fn inode(&self) -> Option<Arc<dyn Inode>> {
       self.inner().inode.lock().as_ref().map(Arc::clone)
    }
    /// set the inode it points to
    fn set_inode(&self, inode: Arc<dyn Inode>) {
        if self.inner().inode.lock().is_some() {
            warn!("[Dentry] trying to replace inode with {:?}", self.name());
        }
        *self.inner().inode.lock() = Some(inode);
    }
    /// get the super block field
    fn superblock(&self) -> Arc<dyn SuperBlock> {
        self.inner().superblock.upgrade().unwrap()
    }
    /// tidier way to get parent
    fn parent(&self) -> Option<Arc<dyn Dentry>> {
        self.inner().parent.as_ref().map(|p| p.upgrade().unwrap())
    }
    /// tider way to get name
    fn name(&self) -> &str {
        &self.inner().name
    }
    /// get the state
    fn state(&self) -> DentryState {
        *self.inner().state.lock()
    }
    /// set the state
    fn set_state(&self, state: DentryState) {
        *self.inner().state.lock() = state;
    }
    /// get the absolute path of the dentry
    fn path(&self) -> String {
        if let Some(p) = self.parent() {
            let p_path = p.path();
            if p_path == "/" {
                p_path + self.name()
            } else {
                p_path + "/" + self.name()
            }
        } else {
            // no parent: at the root
            String::from("/")
        }
    }
    /// walk and search the dentry using the given related path(ex. a/b/c)
    /// construct the dentry tree along the way
    /// walk start from the current entry, recrusivly
    /// once find the target dentry or reach unexisted path, return
    /// if find, should return a USED dentry
    /// if not find, should return a NEGATIVE dentry
    fn walk(self: Arc<Self>, path: &str) -> Arc<dyn Dentry>;
    
}

impl dyn Dentry {
    /// find the dentry by given path
    /// first look up the dcache
    /// if missed, try to search, start from this dentry
    /// only return USED dentry, panic on invalid path
    pub fn find(self: &Arc<Self>, path: &str) -> Option<Arc<dyn Dentry>> {
        // dcache lock must be release before calling other dentry trait
        {
            let cache = DCACHE.lock();
            if let Some(dentry) = cache.get(path) {
                //info!("[DCACHE] hit one: {:?}", dentry.name());
                return Some(dentry.clone());
            }
        }
        //info!("[DCACHE] miss one: {}, start to search from {}", path, self.path());
        let dentry = self.clone().walk(path);
        if dentry.state() == DentryState::NEGATIVE {
            //info!("[DENTRY] invalid path!");
            None
        } else {
            Some(dentry.clone())
        }
    }
}



#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
/// dentry state
pub enum DentryState {
    #[default]
    /// USED: the path is valid
    USED,
    /// UNUSED: maybe init state
    UNUSED,
    /// NEGATIVE: the path is invalid
    NEGATIVE,
}

#[allow(unused)]
/// dcache: dentry cache to speed up dentry looking
/// when open a file, need to add the related dentry to cache
/// when close a file, remove it in the cache
/// every used or negative dentry should be in cache
pub static DCACHE: SpinNoIrqLock<BTreeMap<String, Arc<dyn Dentry>>> = 
    SpinNoIrqLock::new(BTreeMap::new());

// get the dentry by path
// if the inode and its dentry exist, get the dentry
// if the inode not exist but dentry is exist, get the negative dentry
// if the dentry is never created, return None and let the caller to construct a new dentry
// and insert it into the cache
#[allow(unused)]
pub fn get(path: &str) -> Option<Arc<dyn Dentry>> {
    info!("[DCACHE] trying to get {}", path);
    DCACHE.lock().get(path).cloned()
}

// insert a dentry into the cache
#[allow(unused)]
pub fn insert(path: String, dentry: Arc<dyn Dentry>) {
    info!("[DCACHE] insert <{}, {}>", path, dentry.name());
    DCACHE.lock().insert(path, dentry);
}

// remove a dentry in the dache
#[allow(unused)]
pub fn remove(path: &str) -> Option<Arc<dyn Dentry>>{
    DCACHE.lock().remove(path)
}
