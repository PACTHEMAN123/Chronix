//! Pipe file system
//! pipe is at the level of inode
//! every pipe has a Read File and Write File

use core::future::Future;
use core::task::{Poll, Waker};

use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use async_trait::async_trait;
use log::info;

use super::vfs::InodeInner;
use crate::processor::context::SumGuard;
use crate::{mm::UserBuffer, sync::mutex::{SpinNoIrq, SpinNoIrqLock}, utils::RingBuffer};
use crate::fs::vfs::{File, FileInner};

/// a Pipe 
pub struct Pipe {
    write_close: bool,
    read_close: bool,
    ring_buffer: Arc<SpinNoIrqLock<RingBuffer>>,
    read_waker: VecDeque<Waker>,
    write_waker: VecDeque<Waker>,
}

impl Pipe {
    /// create a fix size pipe
    pub fn new(capacity: usize) -> Self {
        Self {
            write_close: false,
            read_close: false,
            ring_buffer: Arc::new(SpinNoIrqLock::new(RingBuffer::new(capacity))),
            read_waker: VecDeque::new(),
            write_waker: VecDeque::new(),
        }
    }
}


/// Pipe File
/// we dont need to use FileInner cuz it dont need a inode
pub struct PipeFile {
    pipe: Arc<SpinNoIrqLock<Pipe>>,
    buffer: Arc<SpinNoIrqLock<RingBuffer>>,
    operate: bool,
}

impl PipeFile {
    fn new(pipe: Arc<SpinNoIrqLock<Pipe>>, is_reader: bool) -> Arc<Self> {
        let buffer = pipe.lock().ring_buffer.clone();
        if is_reader {
            info!("create a new reader");
            Arc::new(Self {
                pipe: pipe.clone(),
                buffer: buffer,
                operate: true,
            })
        } else {
            info!("create a new writer");
            Arc::new(Self {
                pipe: pipe.clone(),
                buffer: buffer,
                operate: false,
            })
        }
    }
}

#[async_trait]
impl File for PipeFile {
    fn file_inner(&self) -> &FileInner {
        panic!("[PipeFile] inner dont exist!");
    }

    fn readable(&self) -> bool {
        self.operate
    }

    fn writable(&self) -> bool {
        !self.operate
    }

    async fn read(&self, buf: &mut [u8]) -> usize {
        assert!(self.readable());
        //info!("[Pipe]: start to read {} bytes", buf.len());
        // create a read future
        let read_size = PipeReadFuture::new(self.buffer.clone(), buf, self.pipe.clone()).await;
        // wake up the writer
        let mut pipe = self.pipe.lock();
        if let Some(waker) = pipe.write_waker.pop_front() {
            waker.wake();
        }
        read_size
    }

    async fn write(&self, buf: &[u8]) -> usize {
        assert!(self.writable());
        //info!("[Pipe]: start to write {} bytes", buf.len());
        // create a write future
        let write_size = PipeWriteFuture::new(self.buffer.clone(), buf, self.pipe.clone()).await;
        // wake up the reader
        //info!("start to wake the reader");
        let mut pipe = self.pipe.lock();
        if let Some(waker) = pipe.read_waker.pop_front() {
            waker.wake();
        }
        write_size
    }
}

impl Drop for PipeFile {
    fn drop(&mut self) {
        let mut pipe = self.pipe.lock();
        if self.operate == true {
            // drop reader
            pipe.read_close = true;
            // wake up all waiting writers
            while let Some(waker) = pipe.write_waker.pop_front() {
                waker.wake();
            }
        } else {
            // drop writer
            pipe.write_close = true;
            // wake up all waiting readers
            while let Some(waker) = pipe.read_waker.pop_front() {
                waker.wake()
            }
        }
    }
}

/// read future
struct PipeReadFuture<'a> {
    buffer: Arc<SpinNoIrqLock<RingBuffer>>,
    user_buf: &'a mut [u8],
    already_put: usize,
    pipe: Arc<SpinNoIrqLock<Pipe>>,
}

impl<'a> PipeReadFuture<'a> {
    pub fn new(
        ringbuf: Arc<SpinNoIrqLock<RingBuffer>>,
        user_buf: &'a mut [u8],
        pipe: Arc<SpinNoIrqLock<Pipe>>,
    ) -> Self {
        Self {
            buffer: ringbuf,
            user_buf: user_buf,
            already_put: 0,
            pipe: pipe,
        }
    }
}

impl<'a> Future for PipeReadFuture<'a> {
    type Output = usize;
    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let _sum_guard = SumGuard::new();

        if self.user_buf.len() == 0 {
            return Poll::Ready(0);
        }

        let this = unsafe { self.get_unchecked_mut() };
        let mut ring_buf = this.buffer.lock();
        // trying to read
        log::debug!("[PipeReadFuture]: read");
        let read_size: usize = ring_buf.read(this.user_buf);
        this.already_put += read_size;
        if read_size == 0 {
            if this.pipe.lock().write_close {
                //info!("[PipeWriteFuture]: all write closed");
                return Poll::Ready(this.already_put);
            } else {
                //info!("[PipeWriteFuture]: nothing to read, waiting");
                this.pipe.lock().read_waker.push_back(cx.waker().clone());
                return Poll::Pending;
            }
        } 
        //info!("[PipeWriteFuture]: write {} in a time", total_read_size);
        return Poll::Ready(this.already_put);
    }
}

/// write future
struct PipeWriteFuture<'a> {
    buffer: Arc<SpinNoIrqLock<RingBuffer>>,
    user_buf: &'a [u8],
    already_put: usize,
    pipe: Arc<SpinNoIrqLock<Pipe>>,
}

impl<'a> PipeWriteFuture<'a> {
    pub fn new(
        ringbuf: Arc<SpinNoIrqLock<RingBuffer>>,
        user_buf: &'a [u8],
        pipe: Arc<SpinNoIrqLock<Pipe>>
    ) -> Self {
        Self {
            buffer: ringbuf,
            user_buf: user_buf,
            already_put: 0,
            pipe: pipe,
        }
    }
}

impl<'a> Future for PipeWriteFuture<'a> {
    type Output = usize;
    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let _sum_guard = SumGuard::new();

        if self.user_buf.len() == 0 {
            return Poll::Ready(0);
        }

        let this = unsafe { self.get_unchecked_mut() };
        let mut ring_buf = this.buffer.lock();
        // trying to write
        //info!("[PipeWriteFuture]: write");
        let write_size: usize = ring_buf.write(this.user_buf);
        this.already_put += write_size;
        if write_size == 0 {
            if this.pipe.lock().read_close {
                //info!("[PipeWriteFuture]: all read closed");
                return Poll::Ready(this.already_put);
            } else {
                //info!("[PipeWriteFuture]: no space to write, waiting");
                this.pipe.lock().write_waker.push_back(cx.waker().clone());
                return Poll::Pending;
            }
        } 
        //info!("[PipeWriteFuture]: write {} in a time", total_write_size);
        return Poll::Ready(this.already_put);
    }
}


/// global function to create a pipe and return the reader and writer file
pub fn make_pipe(capacity: usize) -> (Arc<dyn File>, Arc<dyn File>) {
    let pipe = Arc::new(SpinNoIrqLock::new(Pipe::new(capacity)));
    let read_file = PipeFile::new(pipe.clone(), true);
    let write_file = PipeFile::new(pipe, false);
    (read_file, write_file)
}