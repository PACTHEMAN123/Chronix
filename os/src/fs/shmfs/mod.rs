use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use file::ShmFile;

use crate::{sync::mutex::SpinNoIrqLock, task::TidAllocator};

pub mod inode;
pub mod file;
