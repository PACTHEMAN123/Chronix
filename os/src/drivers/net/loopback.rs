use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc, vec,vec::Vec};
use fatfs::warn;
use smoltcp::phy::{DeviceCapabilities, Medium};
use spin::Lazy;

use crate::devices::{net::{EthernetAddress, NetBufPool, NetBufPtr}, NetBufPtrTrait, NetDevice};

/// A loopback device that sends packets back to the same device.
pub struct LoopbackDevice {
    queue: VecDeque<Vec<u8>>,
}

impl LoopbackDevice {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            queue: VecDeque::with_capacity(512),
        })
    }
}

unsafe impl Send for LoopbackDevice {}
unsafe impl Sync for LoopbackDevice {}

struct LoopbackBuf(Vec<u8>);

impl NetBufPtrTrait for LoopbackBuf {
    fn packet(&self) -> &[u8] {
        self.0.as_slice()
    }
    fn packet_mut (&mut self) -> &mut [u8] {
        &mut self.0
    }
    fn packet_len(&self) -> usize {
        self.0.len()
    }
}
impl NetDevice for LoopbackDevice {
    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut cp = DeviceCapabilities::default();
        cp.max_transmission_unit = 65535;
        cp.max_burst_size = None;
        cp.medium = Medium::Ip;
        cp
    }

    fn receive(&mut self) ->  Box<dyn NetBufPtrTrait> {
        if let Some(buf) = self.queue.pop_front() {
            log::warn!(
                "[NetDriverOps::receive] now receive {} bytes from LoopbackDev.queue",
                buf.len()
            );
            Box::new(LoopbackBuf(buf))
        }else {
            panic!("no rx buffer available, try again");
        }
    }

    fn transmit(&mut self, tx_buf: Box<dyn NetBufPtrTrait>) {
        let data = tx_buf.packet().to_vec();
        log::warn!("[NetDriverOps::transmit] now transmit {} bytes", data.len());
        self.queue.push_back(data);

    }

    fn alloc_tx_buffer(&mut self, size: usize) -> Box<dyn NetBufPtrTrait> {
        let mut buffer = vec![0;size];
        buffer.resize(size, 0);
        Box::new(LoopbackBuf(buffer))
    }

    fn recycle_rx_buffer(&mut self, _rx_buf: Box<dyn NetBufPtrTrait>) {
        unimplemented!()
    }

    fn recycle_tx_buffer(&mut self) {
        unimplemented!()
    }

    fn mac_address(&self) -> crate::devices::net::EthernetAddress {
        EthernetAddress([0; 6])
    }
}
