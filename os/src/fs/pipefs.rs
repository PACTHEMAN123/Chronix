//! pipe file system
//! adapt from phoenix

use core::{future::Future, pin::Pin, task::{Context, Poll, Waker}};

use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use alloc::boxed::Box;
use async_trait::async_trait;

use crate::{fs::StatxTimestamp, sync::mutex::SpinNoIrqLock, syscall::SysError, utils::{get_waker, RingBuffer}};

use super::{vfs::{file::PollEvents, inode::InodeMode, File, FileInner, Inode, InodeInner}, Kstat, Xstat, XstatMask};



pub struct PipeInode {
    inner: InodeInner,
    pipe_meta: SpinNoIrqLock<PipeMeta>
}

pub struct PipeMeta {
    is_write_closed: bool,
    is_read_closed: bool,
    ring_buffer: RingBuffer,
    read_waker: VecDeque<Waker>,
    write_waker: VecDeque<Waker>,
}

impl PipeInode {
    pub fn new(len: usize) -> Arc<Self> {
        let inner = InodeInner::new(Arc::<usize>::new_uninit(), InodeMode::FIFO, len);
        let pipe_meta = SpinNoIrqLock::new(PipeMeta {
            is_write_closed: false,
            is_read_closed: false,
            ring_buffer: RingBuffer::new(len),
            read_waker: VecDeque::new(),
            write_waker: VecDeque::new(),
        });
        Arc::new(Self { inner, pipe_meta })
    }
}

impl Inode for PipeInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn getattr(&self) -> Kstat {
        let inner = self.inode_inner();
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode.bits() as _,
            st_nlink: inner.nlink() as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: inner.size() as _,
            _pad1: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atime_sec: inner.atime().tv_sec as _,
            st_atime_nsec: inner.atime().tv_nsec as _,
            st_mtime_sec: inner.mtime().tv_sec as _,
            st_mtime_nsec: inner.mtime().tv_nsec as _,
            st_ctime_sec: inner.ctime().tv_sec as _,
            st_ctime_nsec: inner.ctime().tv_nsec as _,
        }
    }

    fn getxattr(&self, mask: XstatMask) -> Xstat {
        const SUPPORTED_MASK: XstatMask = XstatMask::from_bits_truncate({
            XstatMask::STATX_BLOCKS.bits |
            XstatMask::STATX_ATIME.bits |
            XstatMask::STATX_CTIME.bits |
            XstatMask::STATX_MTIME.bits |
            XstatMask::STATX_NLINK.bits |
            XstatMask::STATX_MODE.bits |
            XstatMask::STATX_SIZE.bits |
            XstatMask::STATX_INO.bits
        });
        let mask = mask & SUPPORTED_MASK;
        let inner = self.inode_inner();
        Xstat {
            stx_mask: mask.bits,
            stx_blksize: 0,
            stx_attributes: 0,
            stx_nlink: inner.nlink() as u32,
            stx_uid: 0,
            stx_gid: 0,
            stx_mode: inner.mode.bits() as _,
            stx_ino: inner.ino as u64,
            stx_size: inner.size() as _,
            stx_blocks: 0,
            stx_attributes_mask: 0,
            stx_atime: StatxTimestamp {
                tv_sec: inner.atime().tv_sec as _,
                tv_nsec: inner.atime().tv_nsec as _,
            },
            stx_btime: StatxTimestamp {
                tv_sec: 0,
                tv_nsec: 0,
            },
            stx_ctime: StatxTimestamp {
                tv_sec: inner.ctime().tv_sec as _,
                tv_nsec: inner.ctime().tv_nsec as _,
            },
            stx_mtime: StatxTimestamp {
                tv_sec: inner.mtime().tv_sec as _,
                tv_nsec: inner.mtime().tv_nsec as _,
            },
            stx_rdev_major: 0,
            stx_rdev_minor: 0,
            stx_dev_major: 0,
            stx_dev_minor: 0,
            stx_mnt_id: 0,
            stx_dio_mem_align: 0,
            std_dio_offset_align: 0,
            stx_subvol: 0,
            stx_atomic_write_unit_min: 0,
            stx_atomic_write_unit_max: 0,
            stx_atomic_write_segments_max: 0,
            stx_dio_read_offset_align: 0,
        }
    }
}

pub struct PipeWriteFuture {
    events: PollEvents,
    pipe: Arc<PipeInode>
}

impl PipeWriteFuture {
    pub fn new(pipe: Arc<PipeInode>, events: PollEvents) -> Self {
        Self { pipe, events }
    }
}

impl Future for PipeWriteFuture {
    type Output = PollEvents;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut meta = self.pipe.pipe_meta.lock();
        let mut res = PollEvents::empty();
        if meta.is_read_closed {
            res |= PollEvents::ERR;
            return Poll::Ready(res);
        }
        if self.events.contains(PollEvents::OUT) && !meta.ring_buffer.is_full() {
            res |= PollEvents::OUT;
            Poll::Ready(res)
        } else {
            meta.write_waker.push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct PipeReadFuture {
    events: PollEvents,
    pipe: Arc<PipeInode>,
}

impl PipeReadFuture {
    fn new(pipe: Arc<PipeInode>, events: PollEvents) -> Self {
        Self { pipe, events }
    }
}

impl Future for PipeReadFuture {
    type Output = PollEvents;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut meta = self.pipe.pipe_meta.lock();
        let mut res = PollEvents::empty();
        if self.events.contains(PollEvents::IN) && !meta.ring_buffer.is_empty() {
            res |= PollEvents::IN;
            Poll::Ready(res)
        } else {
            if meta.is_write_closed {
                res |= PollEvents::HUP;
                return Poll::Ready(res);
            }
            meta.read_waker.push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct PipeFile {
    pipe: Arc<PipeInode>,
    operate: bool
}

impl PipeFile {
    fn new(inode: Arc<PipeInode>, is_reader: bool) -> Arc<Self> {
        Arc::new(Self {
            pipe: inode,
            operate: is_reader,
        })
    }
}

#[async_trait]
impl File for PipeFile {
    fn file_inner(&self) ->  &FileInner {
        panic!("no inner for pipe file")
    }

    fn readable(&self) -> bool {
        self.operate
    }

    fn writable(&self) -> bool {
        !self.operate
    }

    async fn read(&self, buf: &mut [u8]) -> Result<usize, SysError> {
        assert!(self.operate == true);
        let pipe = self.pipe.clone();
        let events = PollEvents::IN;
        let revents = PipeReadFuture::new(pipe.clone(), events).await;
        if revents.contains(PollEvents::HUP) {
            return Ok(0);
        }
        assert!(revents.contains(PollEvents::IN));
        let mut meta = pipe.pipe_meta.lock();

        log::debug!("reading into buf ptr: {:p}", buf.as_ptr());
        let len = meta.ring_buffer.read(buf);
        if let Some(waker) = meta.write_waker.pop_front() {
            waker.wake();
        }
        return Ok(len);
    }

    async fn write(&self, buf: &[u8]) -> Result<usize, SysError> {
        assert!(self.operate == false);
        let pipe = self.pipe.clone();
        let revents = PipeWriteFuture::new(pipe.clone(), PollEvents::OUT).await;
        if revents.contains(PollEvents::ERR) {
            return Err(SysError::EPIPE);
        }
        assert!(revents.contains(PollEvents::OUT));
        let mut meta = pipe.pipe_meta.lock();
        let len = meta.ring_buffer.write(buf);
        if let Some(waker) = meta.read_waker.pop_front() {
            waker.wake();
        }
        return Ok(len);
    }

    async fn base_poll(&self, events: PollEvents) -> PollEvents {
        if self.operate == false {
            // writer
            let waker = get_waker().await;
            let pipe = self.pipe.clone();
            let mut meta = pipe.pipe_meta.lock();
            let mut res = PollEvents::empty();
            if meta.is_read_closed {
                res |= PollEvents::ERR;
            }
            if events.contains(PollEvents::OUT) && !meta.ring_buffer.is_full() {
                res |= PollEvents::OUT;
            } else {
                meta.write_waker.push_back(waker);
            }
            res
        } else {
            // reader
            let pipe = self.pipe.clone();
            let waker = get_waker().await;
            let mut meta = pipe.pipe_meta.lock();
            let mut res = PollEvents::empty();
            if meta.is_write_closed {
                res |= PollEvents::HUP;
            }
            if events.contains(PollEvents::IN) && !meta.ring_buffer.is_empty() {
                res |= PollEvents::IN;
            } else {
                meta.read_waker.push_back(waker);
            }
            res
        }
    }
}

impl Drop for PipeFile {
    fn drop(&mut self) {
        if self.operate == true {
            let pipe = self.pipe.clone();
            let mut meta = pipe.pipe_meta.lock();
            meta.is_read_closed = true;
            while let Some(waker) = meta.write_waker.pop_front() {
                waker.wake();
            }
        } else {
            let pipe = self.pipe.clone();
            let mut meta = pipe.pipe_meta.lock();
            meta.is_write_closed = true;
            while let Some(waker) = meta.read_waker.pop_front() {
                waker.wake();
            }
        }
    }
}

/// global function to create a pipe and return the reader and writer file
pub fn make_pipe(capacity: usize) -> (Arc<dyn File>, Arc<dyn File>) {
    let pipe = PipeInode::new(capacity);
    let read_file = PipeFile::new(pipe.clone(), true);
    let write_file = PipeFile::new(pipe, false);
    (read_file, write_file)
}