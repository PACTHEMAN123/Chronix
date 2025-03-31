//! VirtIO block device driver

use crate::devices::BlockDevice;
use crate::config::BLOCK_SIZE;
use crate::drivers::dma::VirtioHal;
use crate::sync::UPSafeCell;
use hal::constant::{Constant, ConstantsHal};
use core::ptr::NonNull;

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};
use virtio_drivers::BufferDirection;

use log::*;

const VIRTIO0: usize = 0x10001000 | Constant::KERNEL_ADDR_SPACE.start;

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<VirtioHal, MmioTransport>>);

impl BlockDevice for VirtIOBlock {

    fn size(&self) -> u64 {
        self.0
            .exclusive_access()
            .capacity() * (BLOCK_SIZE as u64)
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
    
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .exclusive_access()
            .read_blocks(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .exclusive_access()
            .write_blocks(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            let header = core::ptr::NonNull::new(VIRTIO0 as *mut VirtIOHeader).unwrap();
            let transport = MmioTransport::new(header, 4096).unwrap();
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal, MmioTransport>::new(transport).expect("failed to create blk driver"),
            ))
        }
    }
}
