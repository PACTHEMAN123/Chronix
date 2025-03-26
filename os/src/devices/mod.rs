#[allow(unused)]
pub mod net;
use core::any::Any;
use alloc::boxed::Box;
use net::NetBufPtr;
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
}

