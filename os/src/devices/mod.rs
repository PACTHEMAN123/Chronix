#[allow(dead_code)]
pub mod net;
use core::any::Any;
use alloc::boxed::Box;
use net::{EthernetAddress, NetBufPtr};
use smoltcp::phy::{DeviceCapabilities,RxToken, TxToken};
/// Trait for block devices
/// which reads and writes data in the unit of blocks

pub trait BlockDevice: Send + Sync + Any {
    fn size(&self) -> u64;

    fn block_size(&self) -> usize;

    /// Read data form block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]);

    /// Write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]);
}

#[allow(unused)]
pub trait NetDevice: Send + Sync + Any {
    // ! smoltcp demands that the device must have below trait
    ///Get a description of device capabilities.
    fn capabilities(&self) -> DeviceCapabilities;
    /// Construct a token pair consisting of one receive token and one transmit token.
    fn receive(&mut self) ->  Box<NetBufPtr>;
    /// Transmits a packet in the buffer to the network, without blocking,
    fn transmit(&mut self, tx_buf: Box<NetBufPtr>); 
    // ! method in implementing a network device concering buffer management
    /// allocate a tx buffer
    fn alloc_tx_buffer(&mut self, size: usize) -> Box<NetBufPtr>;
    /// recycle buf when rx complete
    fn recycle_rx_buffer(&mut self, rx_buf: Box<NetBufPtr>);
    /// recycle used tx buffer
    fn recycle_tx_buffer(&mut self);
    #[allow(dead_code)]
    /// ethernet address of the NIC
    fn mac_address(&self) -> EthernetAddress;
}

