//! contents of kernel folder

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::string::ToString;

use crate::fs::tmpfs::inode::InodeContent;

pub struct PidMax {
    pid_max: AtomicUsize,
}

impl PidMax {
    pub fn new() -> Self {
        Self {
            pid_max: AtomicUsize::new(4194304usize)
        }
    }

    pub fn set_pid_max(&self, new_pid_max: usize) {
        self.pid_max.store(new_pid_max, Ordering::Relaxed);
    }

    pub fn get_pid_max(&self) -> usize {
        self.pid_max.load(Ordering::Relaxed)
    }
}

impl InodeContent for PidMax {
    fn serialize(&self) -> alloc::string::String {
        let size = self.get_pid_max();
        size.to_string()
    }
}



