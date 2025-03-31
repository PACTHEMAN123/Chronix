use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use fatfs::warn;
use smoltcp::phy::{DeviceCapabilities, Medium};

use crate::devices::{net::{EthernetAddress, NetBufPool, NetBufPtr}, NetDevice};

/// A loopback device that sends packets back to the same device.
pub struct LoopbackDevice {
    queue: VecDeque<Box<NetBufPtr>>,
}

impl LoopbackDevice {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            queue: VecDeque::with_capacity(256),
        })
    }
}

unsafe impl Send for LoopbackDevice {}
unsafe impl Sync for LoopbackDevice {}

impl NetDevice for LoopbackDevice {
    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut cp = DeviceCapabilities::default();
        cp.max_transmission_unit = 65535;
        cp.max_burst_size = None;
        cp.medium = Medium::Ip;
        cp
    }

    fn receive(&mut self) ->  Box<crate::devices::net::NetBufPtr> {
        if let Some(buf) = self.queue.pop_front() {
            buf
        }else {
            panic!("no rx buffer available, try again");
        }
    }

    fn transmit(&mut self, tx_buf: Box<crate::devices::net::NetBufPtr>) {
        self.queue.push_back(tx_buf);
    }

    fn alloc_tx_buffer(&mut self, _size: usize) -> Box<crate::devices::net::NetBufPtr> {
        unimplemented!()
    }

    fn recycle_rx_buffer(&mut self, _rx_buf: Box<crate::devices::net::NetBufPtr>) {
        unimplemented!()
    }

    fn recycle_tx_buffer(&mut self) {
        unimplemented!()
    }

    fn mac_address(&self) -> crate::devices::net::EthernetAddress {
        EthernetAddress([0; 6])
    }
}