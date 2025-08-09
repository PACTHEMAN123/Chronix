use core::usize;

use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc, vec,vec::Vec};
use fatfs::{info, warn};
use smoltcp::phy::{DeviceCapabilities, Medium};
use spin::Lazy;

use crate::devices::{net::{EthernetAddress, NetBufPool, NetBuf}, DevError, DevResult, NetBufPtrTrait, NetDevice};

/// A loopback device that sends packets back to the same device.
pub struct LoopbackDevice {
    queue: VecDeque<Vec<u8>>,
}

impl LoopbackDevice {
    pub fn new() -> Box<Self> {
        let inner =Box::new(Self {
            queue: VecDeque::with_capacity(64*1024),
        });
        log::info!("queue length: {} ", inner.queue.len());
        inner
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

    fn receive(&mut self) ->  DevResult<Box<dyn NetBufPtrTrait>> {
        if let Some(buf) = self.queue.pop_front() {
            // log::warn!(
                // "[NetDriverOps::receive] now receive {} bytes from LoopbackDev.queue",
                // buf.len()
            // );
            Ok(Box::new(LoopbackBuf(buf)))
        }else {
            Err(DevError::Again)
        }
    }

    fn transmit(&mut self, tx_buf: Box<dyn NetBufPtrTrait>) -> DevResult {
        let data = tx_buf.packet().to_vec();
        // log::warn!("[Loopback::transmit] now transmit {} bytes", data.len());
        self.queue.push_back(data);
        Ok(())
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<Box<dyn NetBufPtrTrait>> {
        let mut buffer = vec![0;size];
        buffer.resize(size, 0);
        Ok(Box::new(LoopbackBuf(buffer)))
    }

    fn recycle_rx_buffer(&mut self, _rx_buf: Box<dyn NetBufPtrTrait>) -> DevResult{
        Ok(())
    }

    fn recycle_tx_buffer(&mut self) -> DevResult{
        Ok(())
    }

    fn mac_address(&self) -> crate::devices::net::EthernetAddress {
        EthernetAddress([0; 6])
    }

    fn can_transmit(&self) -> bool {
        true
    }
    fn can_receive(&self) -> bool {
        true
    }
    fn rx_queue_size(&self) -> usize {
        usize::MAX
    }
    fn tx_queue_size(&self) -> usize {
        usize::MAX
    }
}
