use core::ptr::NonNull;

use alloc::{boxed::Box, sync::Arc, vec::{self, Vec}};
use log::info;

use crate::sync::mutex::SpinLock;
/// NET_BUF_LEN
pub const NET_BUF_LEN: usize = 1526;
const MIN_BUFFER_LEN: usize = 1526;
const MAX_BUFFER_LEN: usize = 65535;
/// The ethernet address of the NIC (MAC address).
pub struct EthernetAddress(pub [u8; 6]);
/// for each buffer set a NetBufPtr
pub struct NetBufPtr {
    /// the header part bytes length
    header_len: usize,
    /// the packet length
    packet_len: usize,
    /// the whole buffer size
    capacity: usize,
    /// the buffer pointer
    buf_ptr: NonNull<u8>,
    /// the offset to the buffer pool
    pool_offset: usize,
    /// the buffer pool pointer
    pool: Arc<NetBufPool>,
}
/// whole buffer pool
pub struct NetBufPool {
    /// nums of buf
    capacity: usize,
    /// sizeof buf
    buf_len: usize,
    /// the buffer pool
    pool: Vec<u8>,
    /// list of free buffer bytes index
    free_list: SpinLock<Vec<usize>>,
}

impl NetBufPool {
    /// creates a new NetBufPool given the capacity and buffer size
    pub fn new(capacity: usize, buf_len: usize) -> Arc<Self> {
        // buf_len should between MIN_BUFFER_LEN and MAX_BUFFER_LEN
        let pool = alloc::vec![0; capacity * buf_len];
        let mut free_list = Vec::with_capacity(capacity);
        for i in 0..capacity {
            free_list.push(i * buf_len);
        }
        Arc::new(Self {
            capacity,
            buf_len,
            pool,
            free_list: SpinLock::new(free_list),
        })
    }
    /// allocates a new buffer from the pool
    pub fn alloc(self: &Arc<Self>) -> Option<NetBufPtr> {
        let mut free_list = self.free_list.lock();
        if let Some(idx) = free_list.pop() {
            let ptr = NonNull::new(unsafe{self.pool.as_ptr().add(idx) }as *mut u8).unwrap();
            Some(NetBufPtr {
                header_len: 0,
                packet_len: 0,
                capacity: self.buf_len,
                buf_ptr: ptr,
                pool_offset: idx,
                pool: Arc::clone(self),
            })
        } else {
            info!("NetBufPool is full");
            None
        }
    }
    /// allocates a new boxed buffer from the pool
    pub fn alloc_boxed(self: &Arc<Self>) -> Option<NetBufBox> {
        Some(Box::new(self.alloc()?))
    }
    /// deallocates a buffer from the pool, which means mark the idx is free and feel free to write data into it
    pub fn dealloc(&self, idx: usize) {
        self.free_list.lock().push(idx);
    }
}

impl NetBufPtr {
    /// retruns a slice of memory give start and len
    const fn get_slice(&self, start: usize, len: usize) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.buf_ptr.as_ptr().add(start), len)
        }
    }
    /// returns a mutable slice of memory given start and len
    const fn get_mut_slice(&self, start: usize, len: usize) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.buf_ptr.as_ptr().add(start), len)
        }
    }
    /// set length of header
    pub fn set_header_len(&mut self, len: usize) {
        self.header_len = len;
    }
    /// set length of packet]
    pub fn set_packet_len(&mut self, len: usize) {
        self.packet_len = len;
    }
    /// returns header part of the buffer
    pub fn header(&self) -> &[u8] {
        self.get_slice(0, self.header_len)
    }
    /// returns mutable header part of the buffer
    pub fn header_mut(&self) -> &mut [u8] {
        self.get_mut_slice(0, self.header_len)
    }
    /// returns packet part of the buffer
    pub fn packet(&self) -> &[u8] {
        self.get_slice(self.header_len, self.packet_len)
    }
    /// returns mutable packet part of the buffer
    pub fn packet_mut(&self) -> &mut [u8] {
        self.get_mut_slice(self.header_len, self.packet_len)
    }
    /// Returns both the header and the packet parts, as a contiguous slice.
    pub const fn packet_with_header(&self) -> &[u8] {
        self.get_slice(0, self.header_len + self.packet_len) 
    }
    /// returns the whole buffer
    pub fn as_slice(&self) -> &[u8] {
        self.get_slice(0, self.capacity)
    }
    /// returns the whole mutable buffer
    pub fn as_mut_slice(&self) -> &mut [u8] {
        self.get_mut_slice(0, self.capacity)
    }
}

impl Drop for NetBufPtr {
    fn drop(&mut self) {
        self.pool.dealloc(self.pool_offset);
    }
}

pub type NetBufBox = Box<NetBufPtr>;