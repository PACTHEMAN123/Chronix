use core::{future::Future, pin::Pin, task::{Context, Poll, Waker}};
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;
use crate::{fs::vfs::file::PollEvents, net::{socket::Socket, SaFamily}, sync::mutex::SpinNoIrqLock, syscall::{net::SocketType, SysError}, utils::{get_waker, RingBuffer}};


pub struct BufferEndpoint {
    buffer: Mutex<RingBuffer>,
    read_wakers: Mutex<VecDeque<Waker>>,
    write_wakers: Mutex<VecDeque<Waker>>,
}

impl BufferEndpoint {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(RingBuffer::new(capacity)),
            read_wakers: Mutex::new(VecDeque::new()),
            write_wakers: Mutex::new(VecDeque::new()),
        }
    }
}
/// Shared metadata of socketpair,
///  managing communication buffers in both directions.
pub struct SocketPairMeta {
    // end1 -> end2
    end1: BufferEndpoint,
    // end2 -> end1
    end2: BufferEndpoint,
    // Mark whether both ends are closed
    end1_closed: bool,
    end2_closed: bool,
}

/// Internal shared object of socketpair, similar to PipeInode.
pub struct SocketPairInternal {
    meta: SpinNoIrqLock<SocketPairMeta>,
}

impl SocketPairInternal {
    pub fn new(capacity: usize) -> Arc<Self> {
        let meta = SocketPairMeta {
            end1: BufferEndpoint::new(capacity),
            end2: BufferEndpoint::new(capacity),
            end1_closed: false,
            end2_closed: false,
        };
        Arc::new(Self {
            meta: SpinNoIrqLock::new(meta),
        })
    }
}

// Future for reading from socketpair
pub struct SocketPairReadFuture {
    internal: Arc<SocketPairInternal>,
    is_first_end: bool,
}

impl SocketPairReadFuture {
    pub fn new(internal: Arc<SocketPairInternal>, is_first_end: bool) -> Self {
        Self { internal, is_first_end }
    }
}

// --- waiting for reading Future ---
impl Future for SocketPairReadFuture {
    type Output = PollEvents;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut meta = self.internal.meta.lock();
        let (read_endpoint, other_end_closed_mutex) = if self.is_first_end {
            (&meta.end2, &meta.end2_closed)
        } else {
            (&meta.end1, &meta.end1_closed)
        };
        let mut read_buffer = read_endpoint.buffer.lock();

        if !read_buffer.is_empty() {
            return Poll::Ready(PollEvents::IN);
        }
        if *other_end_closed_mutex {
            // The peer is closed and the buffer is empty, triggering HUP
            return Poll::Ready(PollEvents::HUP);
        }
        
        // The buffer is empty and the peer is not closed, register waker and wait
        read_endpoint.read_wakers.lock().push_back(cx.waker().clone());
        Poll::Pending
    }
}

// --- waiting for writing Future ---
pub struct SocketPairWriteFuture {
    internal: Arc<SocketPairInternal>,
    is_first_end: bool,
}

impl SocketPairWriteFuture {
    pub fn new(internal: Arc<SocketPairInternal>, is_first_end: bool) -> Self {
        Self { internal, is_first_end }
    }
}

impl Future for SocketPairWriteFuture {
    type Output = PollEvents;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut meta = self.internal.meta.lock();
        let (write_endpoint, other_end_closed_mutex) = if self.is_first_end {
            (&meta.end1, &meta.end2_closed)
        } else {
            (&meta.end2, &meta.end1_closed)
        };
        let mut write_buffer = write_endpoint.buffer.lock();

        if !write_buffer.is_empty() {
            return Poll::Ready(PollEvents::IN);
        }
        if *other_end_closed_mutex {
            // The peer is closed and the buffer is empty, triggering HUP
            return Poll::Ready(PollEvents::HUP);
        }
        
        // The buffer is empty and the peer is not closed, register waker and wait
        write_endpoint.write_wakers.lock().push_back(cx.waker().clone());
        Poll::Pending
    }
}


// socketpair concerning

pub struct SocketPairConnection {
    pub internal: Arc<SocketPairInternal>,
    pub is_first_end: bool, // tell whether socket1 or socket2
}

impl SocketPairConnection {
    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize, SysError> {
        let revents = SocketPairReadFuture::new(self.internal.clone(), self.is_first_end).await;

        if revents.contains(PollEvents::HUP) {
            return Ok(0); 
        }
        
        let mut meta = self.internal.meta.lock();
        let (mut read_buffer, mut writer_waker) = if self.is_first_end {
            (meta.end2.buffer.lock(), meta.end2.write_wakers.lock())
        } else {
            (meta.end1.buffer.lock(), meta.end1.write_wakers.lock())
        };

        let len = read_buffer.read(buf);
        if len > 0 {
            // Successfully read data and wake up the peer that may be waiting to write
            if let Some(waker) = writer_waker.pop_front() {
                waker.wake();
            }
        }
        Ok(len)
    }

    /// Send data to this endpoint (implement send)
    pub async fn send(&self, buf: &[u8]) -> Result<usize, SysError> {
        let revents = SocketPairWriteFuture::new(self.internal.clone(), self.is_first_end).await;

        if revents.contains(PollEvents::ERR) {
            return Err(SysError::EPIPE); // The peer is closed, the pipe is broken
        }

        assert!(revents.contains(PollEvents::OUT));

        let mut meta = self.internal.meta.lock();
        let (mut write_buffer, mut reader_waker) = if self.is_first_end {
            (meta.end1.buffer.lock(), meta.end1.read_wakers.lock())
        } else {
            (meta.end2.buffer.lock(), meta.end2.read_wakers.lock())
        };

        let len = write_buffer.write(buf);
        if len > 0 {
            // Successfully wrote data and wake up the peer that may be waiting to read
            if let Some(waker) = reader_waker.pop_front() {
                waker.wake();
            }
        }
        Ok(len)
    }

    /// Implementing poll logic
    pub async fn poll(&self, events: PollEvents) -> PollEvents {
        let mut res = PollEvents::empty();
        let waker = get_waker().await;
        let mut meta = self.internal.meta.lock();

        let (mut read_buffer, mut write_buffer, mut read_waker, mut write_waker, other_end_closed) = if self.is_first_end {
            (
                meta.end2.buffer.lock(), meta.end1.buffer.lock(), 
                meta.end2.read_wakers.lock(), meta.end1.write_wakers.lock(),
                meta.end2_closed
            )
        } else {
            (
                (
                meta.end1.buffer.lock(), meta.end2.buffer.lock(), 
                meta.end1.read_wakers.lock(), meta.end2.write_wakers.lock(),
                meta.end1_closed
            )
            )
        };
        
        // Check for read events
        if !read_buffer.is_empty() {
            res |= PollEvents::IN;
        } else if other_end_closed {
            res |= PollEvents::HUP; // The peer is closed and there is no data to read
        }

        // Check for write events
        if other_end_closed {
            res |= PollEvents::ERR; // The peer is closed, and the write operation will fail.
        } else if !write_buffer.is_full() {
            res |= PollEvents::OUT;
        }

        // If the requested event is not currently ready, register the waker
        if events.contains(PollEvents::IN) && !res.contains(PollEvents::IN | PollEvents::HUP) {
            read_waker.push_back(waker.clone());
        }
        if events.contains(PollEvents::OUT) && !res.contains(PollEvents::OUT | PollEvents::ERR) {
            write_waker.push_back(waker);
        }

        res
    }

    /// close the endpoint
    pub fn close(&self) {
        let mut meta = self.internal.meta.lock();
        if self.is_first_end {
            if meta.end1_closed { return; }
            meta.end1_closed = true;
            // Wake up all wakers waiting for end1 to be written (they will receive ERR)
            while let Some(waker) = meta.end1.write_wakers.lock().pop_front() { waker.wake(); }
            // Wake up all wakers waiting for end1 to be read (they will receive HUP)
            while let Some(waker) = meta.end2.read_wakers.lock().pop_front() { waker.wake(); }
        } else {
            if meta.end2_closed { return; }
            meta.end2_closed = true;
            // Wake up all wakers waiting for end2 to be written (they will receive ERR)
            while let Some(waker) = meta.end2.write_wakers.lock().pop_front() { waker.wake(); }
            // Wake up all wakers waiting for end2 to be read (they will receive HUP)
            while let Some(waker) = meta.end1.read_wakers.lock().pop_front() { waker.wake(); }
        }
    }
}

impl Drop for SocketPairConnection {
    fn drop(&mut self) {
        self.close();
    }
}

pub fn make_socketpair(domain: SaFamily, sk_type: SocketType, capacity: usize, non_block: bool, protocol: u8) -> (Arc<Socket>, Arc<Socket>) {
    let internal = SocketPairInternal::new(capacity);

    let conn1 = SocketPairConnection {
        internal: internal.clone(),
        is_first_end: true,
    };

    let mut socket1 = Socket::new(
            domain,
            sk_type,
            non_block,
            protocol
        );
    socket1.sk = super::socket::Sock::SocketPair(conn1);
    
    let conn2 = SocketPairConnection {
        internal: internal,
        is_first_end: false,
    };
    let mut socket2 = Socket::new(
        domain,
        sk_type,
        non_block,
        protocol
    );
    socket2.sk = super::socket::Sock::SocketPair(conn2);

    (Arc::new(socket1), Arc::new(socket2))
}