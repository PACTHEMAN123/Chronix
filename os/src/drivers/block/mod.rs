//! Block device driver

mod virtio_blk;
mod pci_blk;
mod mmio_blk;

use core::sync::atomic::AtomicUsize;

use hal::println;
pub use virtio_blk::VirtIOBlock;
pub use pci_blk::VirtIOPCIBlock;
pub use mmio_blk::VirtIOMMIOBlock;

use alloc::sync::Arc;
use crate::devices::{BlockDevice, DeviceMajor, DEVICE_MANAGER};
use lazy_static::*;

// pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;

// lazy_static! {
//     pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
// }

pub static BLK_ID: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    /// WARNING: should only be called after devices manager finish init
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = {
        /*
        let blk = DEVICE_MANAGER.lock()
        .find_dev_by_major(DeviceMajor::Block)
        .into_iter()
        .map(|device| device.as_blk().unwrap())
        .next()
        .unwrap();
         */

        #[cfg(target_arch="riscv64")]
        let blk = DEVICE_MANAGER.lock()
            .find_dev_by_name("sda0", DeviceMajor::Block)
            .as_blk()
            .unwrap();

        #[cfg(target_arch="loongarch64")]
        let blk = DEVICE_MANAGER.lock()
            .find_dev_by_name("sda1", DeviceMajor::Block)
            .as_blk()
            .unwrap();

        blk.clone()
    };
}

#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}