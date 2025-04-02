use core::{panic, ptr::NonNull};

use alloc::{boxed::Box, sync::Arc, vec::{self, Vec}};
use log::info;
use smoltcp::{phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken}, time::Instant};

use crate::{net::modify_tcp_packet, sync::{mutex::SpinLock, UPSafeCell}};

use super::{DevError, NetBufPtrTrait, NetDevice};
/// NET_BUF_LEN
pub const NET_BUF_LEN: usize = 1526;
const MIN_BUFFER_LEN: usize = 1526;
const MAX_BUFFER_LEN: usize = 65535;
/// The ethernet address of the NIC (MAC address).
pub struct EthernetAddress(pub [u8; 6]);
/// for each buffer set a NetBufPtr
pub struct NetBufPtr {
    /// the header part bytes length
    pub header_len: usize,
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

impl NetBufPtrTrait for NetBufPtr {
    fn packet_len(&self) -> usize {
        self.get_packet_len()
    }
    
    fn packet(&self) -> &[u8] {
        self.packet()
    }
    
    fn packet_mut(&mut self) -> &mut [u8] {
        self.packet_mut()
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
    /// returns the packet length
    pub fn get_packet_len(&self) -> usize {
        self.packet_len
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
    pub fn packet_mut(&mut self) -> &mut [u8] {
        self.get_mut_slice(self.header_len, self.packet_len)
    }
    /// Returns both the header and the packet parts, as a contiguous slice.
    pub const fn packet_with_header(&self) -> &[u8] {
        self.get_slice(0, self.header_len + self.packet_len) 
    }
    /// Returns both the header and the packet parts, as a contiguous mut slice.
    pub const fn packet_with_header_mut(&self) -> &mut [u8] {
        self.get_mut_slice(0, self.header_len + self.packet_len) 
    }
    /// returns the whole buffer
    pub fn as_slice(&self) -> &[u8] {
        self.get_slice(0, self.capacity)
    }
    /// returns the whole mutable buffer
    pub fn as_mut_slice(&self) -> &mut [u8] {
        self.get_mut_slice(0, self.capacity)
    }
    /// returns buffer's header length
    pub fn header_len(&self) -> usize {
        self.header_len
    }
    /// returns buffer's capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for NetBufPtr {
    fn drop(&mut self) {
        self.pool.dealloc(self.pool_offset);
    }
}

pub type NetBufBox = Box<NetBufPtr>;

/// device wrapper for network device
pub struct NetDeviceWrapper {
    /// the inner device wrapped by UPSafeCell
    inner: UPSafeCell<Box<dyn NetDevice>>,
}

impl NetDeviceWrapper {
    /// new a NetDeviceWrapper
    pub fn new(dev: Box<dyn NetDevice>) -> Self {
        Self {
            inner: UPSafeCell::new(dev),
        }
    }
}
/// rx token and tx token needed for smoltcp
pub struct NetRxToken<'a>(&'a UPSafeCell<Box<dyn NetDevice>>, Box<dyn NetBufPtrTrait>);
pub struct NetTxToken<'a>(&'a UPSafeCell<Box<dyn NetDevice>>);

impl <'a> RxToken for NetRxToken<'a> {
    /// receive a packet than call the closure with the packet bytes
    fn consume<R, F>(self, f: F) -> R
        where
            F: FnOnce(&[u8]) -> R 
    {
        // need preprocess
        let mut rx_buf = self.1;
        let result = f(rx_buf.packet_mut());
        self.0.exclusive_access().recycle_rx_buffer(rx_buf).unwrap();
        result
    }

    fn preprocess(&self, sockets: &mut smoltcp::iface::SocketSet<'_>) {
        let medium = self.0.exclusive_access().capabilities().medium;
        let is_ethernet = medium==Medium::Ethernet;
        modify_tcp_packet(self.1.packet(),sockets,is_ethernet).ok();
    }
}

impl <'a> TxToken for NetTxToken<'a> {
    /// construct a transmit buffer of size `len` and call the passed closure `f` with a mutable reference to that buffer.
    fn consume<R, F>(self, len: usize, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R 
    {
        let mut tx_buf = self.0.exclusive_access().alloc_tx_buffer(len).unwrap();
        let result = f(tx_buf.packet_mut());
        self.0.exclusive_access().transmit(tx_buf).unwrap();
        result
    }
}
/// Device implementation in Smoltcp
impl Device for NetDeviceWrapper {
    type RxToken<'a> = NetRxToken<'a> where Self: 'a;
    type TxToken<'a> = NetTxToken<'a> where Self: 'a;

    fn capabilities(&self) -> DeviceCapabilities {
        self.inner.get_ref().capabilities()
    }
    fn receive(&mut self, _: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let inner = self.inner.exclusive_access();
        if let Err(e) = inner.recycle_tx_buffer(){
            log::warn!("recycle_tx_buffers failed: {:?}", e);
            return None;
        };
        let rx_buf = match inner.receive(){
            Ok(buf) => buf,
            Err(e) => {
                if !matches!(e, DevError::Again){
                    log::warn!("received failed!, Error: {:?}",e);
                }
                return None;
            }
        };
        Some((NetRxToken(&self.inner, rx_buf), NetTxToken(&self.inner)))
    }
    fn transmit(&mut self, _: Instant) -> Option<Self::TxToken<'_>> {
        let inner = self.inner.exclusive_access();
        match inner.recycle_tx_buffer(){
            Err(e) => {
                log::warn!("[transmit] recycle buffer failed: {:?}",e );
                return None;    
            }
            Ok(_) => {
                Some(NetTxToken(&self.inner))
            },
        }
    }
}