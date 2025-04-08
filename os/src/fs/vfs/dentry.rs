//! virtual file system dentry

use core::default;

use crate::{fs::{vfs::{dentry, inode::InodeMode}, OpenFlags}, sync::mutex::SpinNoIrqLock, syscall::SysError};

use super::{superblock, File, Inode, SuperBlock};

use alloc::{
    collections::btree_map::BTreeMap, string::{String, ToString}, sync::{Arc, Weak}, vec::Vec
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
    /// construct a new Self type dentry
    fn new(
        &self,
        name: &str,
        superblock: Arc<dyn SuperBlock>,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry>;
    /// open the inode it points as File
    fn open(self: Arc<Self>, _flags: OpenFlags) -> Option<Arc<dyn File>> {
        todo!()
    }
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
    /// clear the inode, now it doesnt have a inode
    fn clear_inode(&self) {
        *self.inner().inode.lock() = None;
        self.set_state(DentryState::NEGATIVE);
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
}

impl dyn Dentry {
    
    /// find the dentry by given path
    /// first look up the dcache
    /// if missed, try to search, start from this dentry
    /// only return USED dentry, panic on invalid path
    pub fn find(self: &Arc<Self>, path: &str) -> Option<Arc<dyn Dentry>> {
        // the path should be relative!
        let path = path.trim_start_matches("/");
        // dcache lock must be release before calling other dentry trait
        {
            let cache = DCACHE.lock();
            let abs_path = self.path() + path;
            //info!("[DCACHE] try to get {}", abs_path);
            if let Some(dentry) = cache.get(&abs_path) {
                //info!("[DCACHE] hit one: {:?}", dentry.name());
                if dentry.state() == DentryState::NEGATIVE {
                    return None;
                } else {
                    return Some(dentry.clone());
                }  
            }
        }
        //info!("[DCACHE] miss one: {:?}, start to search from {}", path, self.path());
        let dentry = self.clone().walk(path);
        if dentry.state() == DentryState::NEGATIVE {
            //info!("[DENTRY] invalid path!");
            None
        } else {
            Some(dentry.clone())
        }
    }

    /// walk and search the dentry using the given related path(ex. a/b/c)
    /// construct the dentry tree along the way
    /// walk start from the current entry, recrusivly
    /// once find the target dentry or reach unexisted path, return
    /// if find, should return a USED dentry
    /// if not find, should return a NEGATIVE dentry
    pub fn walk(self: Arc<Self>, path: &str) -> Arc<dyn Dentry> {
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
                let path_dentry = self.new(
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
                let neg_dentry = self.new(
                    name,
                    self.superblock(),
                    Some(current_dentry)
                );
                neg_dentry.set_state(DentryState::NEGATIVE);
                //info!("[DCACHE]: insert key: {}", neg_dentry.path());
                DCACHE.lock().insert(neg_dentry.path(), neg_dentry.clone());
                return neg_dentry;
            }
        }
        return current_dentry.clone();
    }

    /// get all child dentry
    /// we assert that only dir dentry will call this method
    /// it will insert into DACHE by the way
    pub fn child_dentry(self: Arc<Self>) -> Vec<Arc<dyn Dentry>> {
        //info!("in child dentry, under: {}", self.path());
        let inode = self.inode().unwrap().clone();
        let mut child_dentrys: Vec<Arc<dyn Dentry>> = Vec::new();
        for name in inode.ls() {
            let child_inode = inode.lookup(&name).unwrap();
            
            let child_dentry = self.new(
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

    /// follow the link and jump until reach the first NOT link Inode or reach the max depth
    pub fn follow(self: Arc<Self>) -> Result<Arc<dyn Dentry>, SysError> {
        const MAX_LINK_DEPTH: usize = 40;
        let mut current = self.clone();

        for _ in 0..MAX_LINK_DEPTH {
            if current.state() == DentryState::NEGATIVE {
                return Ok(current)
            }

            match current.inode().unwrap().inner().mode {
                InodeMode::LINK => {
                    // follow to the next
                    let path =  current.inode().unwrap().readlink()?;
                    let new_dentry = global_find_dentry(&path);
                    current = new_dentry;
                }
                _ => return Ok(current)
            }
        }
        Err(SysError::ELOOP)
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
/// the key is the absolute path of the dentry
/// the value is the dentry
pub static DCACHE: SpinNoIrqLock<BTreeMap<String, Arc<dyn Dentry>>> = 
    SpinNoIrqLock::new(BTreeMap::new());


/// helper function: Search from root using absolute path,
/// return the target dentry: maybe negative
/// first lookup in the dcache
/// if not found, search from root
pub fn global_find_dentry(path: &str) -> Arc<dyn Dentry> {
    log::info!("global find dentry: {}", path);
    {
        let cache = DCACHE.lock();
        if let Some(dentry) = cache.get(path) {
            return dentry.clone();
        }
    }
    // get the root dentry
    let root_dentry = {
        let dcache = DCACHE.lock();
        Arc::clone(dcache.get("/").unwrap())
    };
    root_dentry.walk(path)
}
