//! virtual file system dentry

use core::{default, mem::MaybeUninit};

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
    /// parent
    pub parent: Option<Weak<dyn Dentry>>,
    /// children
    /// in the case of mount a fs under another fs
    /// we cannot get the child using inode
    /// thats why we need children field
    pub children: SpinNoIrqLock<BTreeMap<String, Arc<dyn Dentry>>>,
    /// state
    pub state: SpinNoIrqLock<DentryState>,
}

impl DentryInner {
    /// create a unused dentry: no children in it
    pub fn new(
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Self {
        let inode = SpinNoIrqLock::new(None);
        Self {
            name: name.to_string(),
            inode,
            parent: parent.map(|p| Arc::downgrade(&p)),
            children: SpinNoIrqLock::new(BTreeMap::new()),
            state: SpinNoIrqLock::new(DentryState::UNUSED),
        }
    }
}

/// dentry method that all fs need to implement
pub trait Dentry: Send + Sync {
    /// get the inner dentry
    fn dentry_inner(&self) -> &DentryInner;
    /// construct a new Self type dentry
    fn new(
        &self,
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry>;
    /// open the inode it points as File
    fn open(self: Arc<Self>, _flags: OpenFlags) -> Option<Arc<dyn File>> {
        todo!()
    }
    /// get the inode it points to
    fn inode(&self) -> Option<Arc<dyn Inode>> {
       self.dentry_inner().inode.lock().as_ref().map(Arc::clone)
    }
    /// set the inode it points to
    fn set_inode(&self, inode: Arc<dyn Inode>) {
        log::debug!("dentry: {} set inode", self.path());
        if self.dentry_inner().inode.lock().is_some() {
            warn!("[Dentry] trying to replace inode with {:?}", self.name());
        }
        *self.dentry_inner().inode.lock() = Some(inode);
        *self.dentry_inner().state.lock() = DentryState::USED;
    }
    /// clear the inode, now it doesnt have a inode
    fn clear_inode(&self) {
        log::debug!("dentry: {} clear inode", self.path());
        *self.dentry_inner().inode.lock() = None;
        self.set_state(DentryState::NEGATIVE);
    }
    /// tidier way to get parent
    fn parent(&self) -> Option<Arc<dyn Dentry>> {
        self.dentry_inner().parent.as_ref().map(|p| p.upgrade().unwrap())
    }
    /// get all children
    fn children(&self) -> BTreeMap<String, Arc<dyn Dentry>> {
        self.dentry_inner().children.lock().clone()
    }
    /// get a child
    fn get_child(&self, name: &str) -> Option<Arc<dyn Dentry>> {
        self.dentry_inner().children.lock().get(name).cloned()
    }
    /// add a child
    fn add_child(&self, child: Arc<dyn Dentry>) {
        self.dentry_inner().children.lock().insert(child.name().to_string(), child);
    }
    /// remove a child
    fn remove_child(&self, name: &str) {
        self.dentry_inner().children.lock().remove(name);
    }
    /// tider way to get name
    fn name(&self) -> &str {
        &self.dentry_inner().name
    }
    /// get the state
    fn state(&self) -> DentryState {
        *self.dentry_inner().state.lock()
    }
    /// set the state
    fn set_state(&self, state: DentryState) {
        *self.dentry_inner().state.lock() = state;
    }
    /// determine if negative dentry
    fn is_negative(&self) -> bool {
        *self.dentry_inner().state.lock() == DentryState::NEGATIVE
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
    /// load all child dentry 
    /// can also be use to update
    /// since the on-disk fs dentry dont know child until lookup by inode
    /// we assert that only dir dentry will call this method
    /// it will insert into DCACHE by the way
    fn load_child_dentry(self: Arc<Self>) -> Result<Vec<Arc<dyn Dentry>>, SysError> {
        Err(SysError::ENOTDIR)
    }
    /// create a negative child which share the same type with self
    fn new_neg_dentry(self: Arc<Self>, _name: &str) -> Arc<dyn Dentry> {
        todo!()
    }
}

impl dyn Dentry {
    
    /// find the dentry by given path
    /// first look up the dcache
    /// if missed, try to search, start from this dentry
    /// only return USED dentry, panic on invalid path
    pub fn find(self: &Arc<Self>, path: &str) -> Result<Option<Arc<dyn Dentry>>, SysError> {
        // the path should be relative!
        let path = path.trim_start_matches("/");
        let mut current = self.clone();
        if path.is_empty() {
            return Ok(Some(current))
        }
        log::info!("path {}", path);
        let normalize_path = {
            let mut compoents = Vec::new();
            for compoent in path.split("/") {
                match compoent {
                    "" | "." => continue,
                    ".." => {
                        current = current.parent().ok_or(SysError::ENOENT)?;
                    }
                    name => {
                        compoents.push(name);
                    }
                }
            }

            compoents.join("/")
        };
        log::info!("normalize path: {}", normalize_path);

        // dcache lock must be release before calling other dentry trait
        {
            let cache = DCACHE.lock();
            let abs_path = current.path() + &normalize_path;
            //info!("[DCACHE] try to get {}", abs_path);
            if let Some(dentry) = cache.get(&abs_path) {
                //info!("[DCACHE] hit one: {:?}", dentry.name());
                if dentry.state() == DentryState::NEGATIVE {
                    return Ok(None);
                } else {
                    return Ok(Some(dentry.clone()));
                }  
            }
        }
        //info!("[DCACHE] miss one: {:?}, start to search from {}", path, self.path());
        let dentry = current.clone().walk(path)?;
        if dentry.state() == DentryState::NEGATIVE {
            //info!("[DENTRY] invalid path!");
            Ok(None)
        } else {
            Ok(Some(dentry.clone()))
        }
    }

    /// walk and search the dentry using the given related path(ex. a/b/c)
    /// construct the dentry tree along the way
    /// walk start from the current entry, recrusivly
    /// once find the target dentry or reach unexisted path, return
    /// if find, should return a USED dentry
    /// if not find, should return a NEGATIVE dentry
    pub fn walk(self: Arc<Self>, path: &str) -> Result<Arc<dyn Dentry>, SysError> {
        let mut current_dentry = self.clone();
        // break down the path: string a/b/c -> vec [a, b, c]
        let name_vec: Vec<&str> = path
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        // use the vec to walk, loop
        // if the element exist, keeping walking
        // if not exist, stop.
        for name in name_vec.iter() {
            if let Some(child_dentry) = current_dentry.get_child(name) {
                // first look into self children field
                // if find, just keep walking
                current_dentry = child_dentry;
            } else {
                // not found, try to update the children
                current_dentry.clone().load_child_dentry()?;
                if let Some(child_dentry) = current_dentry.get_child(name) {
                    // after update find child
                    current_dentry = child_dentry;
                } else {
                    // child not exist
                    // create a negative dentry
                    // WARNING: the neg dentry and its parent should have same types
                    // let neg_dentry = self.new(
                    //     name,
                    //     self.superblock(),
                    //     Some(current_dentry)
                    // );
                    // neg_dentry.set_state(DentryState::NEGATIVE);
                    let neg_dentry = current_dentry.new_neg_dentry(name);
                    //info!("[DCACHE]: insert key: {}", neg_dentry.path());
                    DCACHE.lock().insert(neg_dentry.path(), neg_dentry.clone());
                    return Ok(neg_dentry);
                }
            }
        }

        return Ok(current_dentry.clone());
    }

    /// follow the link and jump until reach the first NOT link Inode or reach the max depth
    pub fn follow(self: Arc<Self>) -> Result<Arc<dyn Dentry>, SysError> {
        const MAX_LINK_DEPTH: usize = 40;
        let mut current = self.clone();

        for _ in 0..MAX_LINK_DEPTH {
            if current.state() == DentryState::NEGATIVE {
                return Ok(current)
            }

            let mode = current.inode().unwrap().inode_inner().mode;
            // log::info!("[walk] mode {:?}", mode);
            if mode.contains(InodeMode::LINK) {
                // follow to the next
                let path =  current.inode().unwrap().readlink()?;
                let new_dentry = global_find_dentry(&path)?;
                current = new_dentry;
            } else {
                return Ok(current)
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
pub fn global_find_dentry(path: &str) -> Result<Arc<dyn Dentry>, SysError> {
    log::debug!("global find dentry: {}", path);
    {
        let cache = DCACHE.lock();
        if let Some(dentry) = cache.get(path) {
            return Ok(dentry.clone());
        }
    }
    // get the root dentry
    let root_dentry = {
        let dcache = DCACHE.lock();
        Arc::clone(dcache.get("/").unwrap())
    };
    root_dentry.walk(path)
}

impl<T: Send + Sync + 'static> Dentry for MaybeUninit<T> {
    fn dentry_inner(&self) -> &DentryInner {
        todo!()
    }

    fn new(
        &self,
        _name: &str,
        _parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        todo!()
    }
}