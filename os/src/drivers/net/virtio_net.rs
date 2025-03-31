use crate::devices::{net::{NetBufBox, NetBufPool, NetBufPtr, NET_BUF_LEN}, NetDevice};
use log::info;
use smoltcp::phy::{DeviceCapabilities, Medium};
use crate::drivers::dma::VirtioHal;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
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
    pub fn new(transport: T) -> Box<Self> {
        const NONE_BUF: Option<NetBufBox> = None;
        let rx_buf = [NONE_BUF; NET_QUEUE_SIZE];
        let tx_buf = [NONE_BUF; NET_QUEUE_SIZE]; 
        let free_tx_bufs = Vec::with_capacity(NET_QUEUE_SIZE); 
        let buf_pool = NetBufPool::new( 2*NET_QUEUE_SIZE, NET_BUF_LEN);
        let raw = VirtIONetRaw::new(transport).unwrap();
        let mut inner_self = Self {
            rx_buffers: rx_buf,
            tx_buffers: tx_buf, 
            free_tx_bufs,
            buf_pool: buf_pool,
            raw_device: raw,
        };
        // for rx_buffer: allocate all
        for (_i,rx_buf_place) in inner_self.rx_buffers.iter_mut().enumerate() {
            let rx_buf = inner_self.buf_pool.alloc_boxed().unwrap();
            unsafe{inner_self.raw_device.
                receive_begin(rx_buf.as_mut_slice()).unwrap()
            };
            *rx_buf_place = Some(rx_buf);
        } 
        // allocate tx_buffers
        for _i in 0..NET_QUEUE_SIZE {
            let mut tx_buf = inner_self.buf_pool.alloc_boxed().unwrap();
            // fill header
            let head_len = inner_self.raw_device.fill_buffer_header(tx_buf.as_mut_slice()).unwrap();
            tx_buf.set_header_len(head_len);
            inner_self.free_tx_bufs.push(tx_buf);
        }
        Box::new(inner_self)
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
    fn receive(&mut self) ->  Box<NetBufPtr> {
        if let Some(token) = self.raw_device.poll_receive() {
            let mut rx_buf = self.rx_buffers[token as usize]
            .take().unwrap();
            let (head_len, packet_len) = unsafe {
                self.raw_device
                .receive_complete(token, rx_buf.as_mut_slice())
                .unwrap()
            };
            rx_buf.set_header_len(head_len);
            rx_buf.set_packet_len(packet_len);
            rx_buf
        }else {
            panic!("no rx buffer available, try again");
        }
    }
    fn transmit(&mut self, tx_buf: Box<NetBufPtr>) {
        let token = unsafe {
            self.raw_device.transmit_begin(tx_buf.packet_with_header()).unwrap()
        };
        self.tx_buffers[token as usize] = Some(tx_buf);
    }
     /// alocate a tx buffer
    fn alloc_tx_buffer(&mut self, size: usize) -> Box<NetBufPtr> {
        let mut net_buf = self.free_tx_bufs.pop().unwrap();
        let packet_len = size;

        let head_len = net_buf.header_len();
        if packet_len + head_len > net_buf.capacity() {
            panic!("tx buffer too small");
        }
        net_buf.set_packet_len(packet_len);
        net_buf
    }
    ///recycle buf when rx complete
    fn recycle_rx_buffer(&mut self, rx_buf: Box<NetBufPtr>) {
        let new_token = unsafe {
            self.raw_device.receive_begin(rx_buf.as_mut_slice())
        }
        .unwrap();
        self.rx_buffers[new_token as usize] = Some(rx_buf);
    }
    /// recycle used tx buffer
    fn recycle_tx_buffer(&mut self) {
        while let Some(token) = self.raw_device.poll_transmit() {
            let tx_buf = self.tx_buffers[token as usize].take().unwrap();
            unsafe {
                let __= self.raw_device.transmit_complete(token, tx_buf.packet_with_header());
            };
            self.free_tx_bufs.push(tx_buf);
        }
    }
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.raw_device.mac_address())
    }
 }
