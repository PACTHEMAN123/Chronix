use core::any::Any;

use crate::devices::{as_dev_err, net::{NetBuf, NetBufBox, NetBufPool, NetBufPtr, NET_BUF_LEN}, DevError, DevResult, NetBufPtrTrait, NetDevice};
use fatfs::warn;
use log::info;
use smoltcp::phy::{DeviceCapabilities, Medium};
use crate::drivers::dma::VirtioHal;
use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use smoltcp::phy::Device;
use virtio_drivers::{
    device::net::VirtIONetRaw,
    transport::{mmio::MmioTransport, Transport},
};
use crate::devices::net::EthernetAddress;

pub const NET_QUEUE_SIZE: usize = 32;
pub struct VirtIoNetDev<T: Transport> {
    rx_buffers: [Option<NetBufBox>; NET_QUEUE_SIZE],
    tx_buffers: [Option<NetBufBox>; NET_QUEUE_SIZE],
    free_tx_bufs: Vec<NetBufBox>,
    buf_pool: Arc<NetBufPool>,
    raw_device: VirtIONetRaw<VirtioHal, T, NET_QUEUE_SIZE>,
}

unsafe impl<T: Transport> Send for VirtIoNetDev<T> {}
unsafe impl<T: Transport> Sync for VirtIoNetDev<T> {}
impl<T: Transport> VirtIoNetDev<T> {
    /// new a VirtIoNetDev
    pub fn new(transport: T) -> DevResult<Box<Self>> {
        const NONE_BUF: Option<NetBufBox> = None;
        let rx_buf = [NONE_BUF; NET_QUEUE_SIZE];
        let tx_buf = [NONE_BUF; NET_QUEUE_SIZE]; 
        let free_tx_bufs = Vec::with_capacity(NET_QUEUE_SIZE); 
        let buf_pool = NetBufPool::new( 2*NET_QUEUE_SIZE, NET_BUF_LEN)?;
        let raw = VirtIONetRaw::new(transport).map_err(as_dev_err)?;
        let mut inner_self = Self {
            rx_buffers: rx_buf,
            tx_buffers: tx_buf, 
            free_tx_bufs,
            buf_pool: buf_pool,
            raw_device: raw,
        };
        // for rx_buffer: allocate all
        for (i,rx_buf_place) in inner_self.rx_buffers.iter_mut().enumerate() {
            let mut rx_buf = inner_self.buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
            // log::warn!("[VirtioNetDev::new] Ok to allocate memory to rx buffer");
            let token = unsafe{inner_self.raw_device
                .receive_begin(rx_buf.as_mut_slice())
                .map_err(as_dev_err)?
            };
            assert_eq!(token, i as u16);
            *rx_buf_place = Some(rx_buf);
        } 
        // allocate tx_buffers
        for _i in 0..NET_QUEUE_SIZE {
            let mut tx_buf = inner_self.buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
            // fill header
            let head_len = inner_self.raw_device.fill_buffer_header(tx_buf.as_mut_slice()).or(Err(DevError::InvalidParam))?;
            tx_buf.set_header_len(head_len);
            inner_self.free_tx_bufs.push(tx_buf);
        }
        Ok(Box::new(inner_self))
    }
 } 

 impl<T: Transport + 'static> NetDevice for VirtIoNetDev<T> {
    /// For Ethernet devices, this is the maximum Ethernet frame size, including
    /// the Ethernet header (14 octets), but *not* including the Ethernet
    /// FCS (4 octets). Therefore, Ethernet MTU = IP MTU + 14.
    /// Note that in Linux and other OSes, "MTU" is the IP MTU, not the Ethernet
    /// MTU, even for Ethernet devices. 
    /// Most common IP MTU is 1500. Minimum is 576 (for IPv4) or 1280 (for
    /// IPv6). Maximum is 9216 octets.
    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = 1514;
        cap.max_burst_size = None;
        cap.medium = Medium::Ethernet;
        cap
    }
    fn receive(&mut self) ->  DevResult<Box<dyn NetBufPtrTrait>> {
        if let Some(token) = self.raw_device.poll_receive() {
            log::warn!("[VirtioNetDev::receive] token {}", token);
            log::warn!("[VirtioNetDev::receive] rx_buffers: {:?}", &self.rx_buffers[token as usize]);
            let mut rx_buf = match self.rx_buffers[token as usize]
            .take() {
                Some(rx_buf) => {
                    log::warn!("[VirtioNetDev::receive] Ok to take rx buffer");
                    rx_buf
                },
                None => {
                    return Err(DevError::BadState);
                }
            };
            let (head_len, packet_len) = unsafe {
                self.raw_device
                .receive_complete(token, rx_buf.as_mut_slice())
                .map_err(as_dev_err)?
            };
            log::info!("[VirtioNetDev::receive] now set packet len {}, head len {}", packet_len, head_len);
            rx_buf.set_header_len(head_len);
            rx_buf.set_packet_len(packet_len);
            Ok(rx_buf.into_buf_ptr())
        }else {
            Err(DevError::Again)
        }
    }
    fn transmit(&mut self, tx_buf: Box<dyn NetBufPtrTrait>) -> DevResult{
        let tx_buf =
            unsafe { core::mem::transmute::<Box<dyn NetBufPtrTrait>, Box<dyn Any + Send>>(tx_buf) };
        let tx_buf = unsafe {
            NetBuf::from_buf_ptr(tx_buf.downcast::<NetBufPtr>().unwrap())
        };
        let token = unsafe {
            self.raw_device.transmit_begin(tx_buf.packet_with_header()).map_err(as_dev_err)?
        };
    log::warn!("[VirtioNetDev::transmit] head_len {}, packet len {}",tx_buf.header_len(),tx_buf.get_packet_len() );
        self.tx_buffers[token as usize] = Some(tx_buf);
        Ok(())
    }
     /// alocate a tx buffer
    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<Box<dyn NetBufPtrTrait>> {
        let mut net_buf = self.free_tx_bufs.pop().ok_or(DevError::NoMemory)?;
        let packet_len = size;
        let head_len = net_buf.header_len();
        if packet_len + head_len > net_buf.capacity() {
            log::warn!("tx buffer too small");
            return Err(DevError::InvalidParam);
        }
        net_buf.set_packet_len(packet_len);
        log::warn!("VirtioNetDev::alloc_tx_buffer: Ok to allocate tx buffer, head_len: {head_len}, packet_len: {packet_len}");
        Ok(net_buf.into_buf_ptr())
    }
    ///recycle buf when rx complete
    fn recycle_rx_buffer(&mut self, rx_buf: Box<dyn NetBufPtrTrait>) -> DevResult {
        let rx_buf_ptr = unsafe {
            core::mem::transmute::<Box<dyn NetBufPtrTrait>, Box<dyn Any + Send>>(rx_buf)
        };
        let mut rx_buf = unsafe {
            NetBuf::from_buf_ptr(rx_buf_ptr.downcast::<NetBufPtr>().unwrap())
        };
        let new_token = unsafe {
            self.raw_device.receive_begin(rx_buf.as_mut_slice())
        }
        .map_err(as_dev_err)?;
        log::warn!("[VirtioNetDev::recycle_rx_buffer] new_token {}", new_token);
        if self.rx_buffers[new_token as usize].is_some() {
            log::warn!("rx buffer already in use");
            return Err(DevError::BadState);
        }
        self.rx_buffers[new_token as usize] = Some(rx_buf);
        log::warn!("[VirtioNetDev] Ok to recycle rx buffer");
        Ok(())
    }
    /// recycle used tx buffer
    fn recycle_tx_buffer(&mut self) -> DevResult {
        while let Some(token) = self.raw_device.poll_transmit() {
            let tx_buf = self.tx_buffers[token as usize].take().ok_or(DevError::BadState)?;
            unsafe {
                self.raw_device.transmit_complete(token, tx_buf.packet_with_header()).map_err(as_dev_err)?;
            };
            self.free_tx_bufs.push(tx_buf);
        }
        log::warn!("[VirtioNetDev] Ok to recycle tx buffer");
        Ok(())
    }
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.raw_device.mac_address())
    }
    fn can_transmit(&self) -> bool {
        !self.free_tx_bufs.is_empty() && self.raw_device.can_send()
    }
    fn can_receive(&self) -> bool {
        self.raw_device.poll_receive().is_some()
    }
    fn rx_queue_size(&self) -> usize {
        NET_QUEUE_SIZE
    }
    fn tx_queue_size(&self) -> usize {
        NET_QUEUE_SIZE
    }
 }

 pub type VirtIoNetDevImpl = VirtIoNetDev<MmioTransport>;