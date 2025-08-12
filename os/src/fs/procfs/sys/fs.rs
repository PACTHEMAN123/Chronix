use core::{cmp, sync::atomic::{AtomicUsize, Ordering}};

use alloc::string::ToString;

use crate::{config::PAGE_SIZE, fs::tmpfs::inode::InodeContent, syscall::PIPE_BUF_LEN};



pub struct PipeMaxSize {
    pipe_max_size: AtomicUsize,
}

impl PipeMaxSize {
    pub fn new() -> Self {
        Self {
            pipe_max_size: AtomicUsize::new(PIPE_BUF_LEN)
        }
    }

    pub fn get(&self) -> usize {
        self.pipe_max_size.load(Ordering::Relaxed)
    }

    pub fn set(&self, size: usize) {
        let size = cmp::max(PAGE_SIZE, size);
        self.pipe_max_size.store(size, Ordering::Relaxed);
    }
}

impl InodeContent for PipeMaxSize {
    fn serialize(&self) -> alloc::string::String {
        self.get().to_string()
    }
}