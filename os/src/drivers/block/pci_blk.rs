//! VirtIO Block using PCI transport
//! 
use core::sync::atomic::Ordering;

use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use hal::addr::PhysPageNumHal;
use hal::allocator::FrameAllocatorHal;
use hal::constant::{Constant, ConstantsHal};
use lazy_static::lazy_static;
use virtio_drivers::transport::pci::bus::{BarInfo, Cam, Command, MemoryBarType, MmioCam, PciRoot};

use crate::config::BLOCK_SIZE;
use crate::devices::pci::{PciDeviceClass, PciDeviceDescriptor};
use crate::devices::{BlockDevice, DevId, Device, DeviceMajor, DeviceMeta};
use crate::drivers::dma::VirtioHal;
use crate::sync::UPSafeCell;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::pci::PciTransport;
use virtio_drivers::transport::{DeviceType, Transport};
use virtio_drivers::BufferDirection;

use super::BLK_ID;

pub struct VirtIOPCIBlock {
    meta: DeviceMeta,
    blk: UPSafeCell<VirtIOBlk<VirtioHal, PciTransport>>,
}

impl BlockDevice for VirtIOPCIBlock {

    fn size(&self) -> u64 {
        self.blk
            .exclusive_access()
            .capacity() * (BLOCK_SIZE as u64)
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
    
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.blk
            .exclusive_access()
            .read_blocks(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.blk
            .exclusive_access()
            .write_blocks(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

impl Device for VirtIOPCIBlock {
    fn meta(&self) -> &DeviceMeta {
        &self.meta
    }

    fn init(&self) {
        // todo!
    }

    fn handle_irq(&self) {
        // todo!
    }

    fn as_blk(self: Arc<Self>) -> Option<Arc<dyn BlockDevice>> {
        Some(self)
    }
}

impl VirtIOPCIBlock {
    /// create a new Virt IO PCI drive Block device
    /// start: PCI memory space start addr
    /// size: PCI memory space size
    pub fn new(pci_dev: PciDeviceDescriptor) -> Self {
        let blk = UPSafeCell::new(
            VirtIOBlk::<VirtioHal, PciTransport>::new(pci_dev.transport.unwrap()).expect("failed to create blk driver"),
        );
        let id = BLK_ID.fetch_add(1, Ordering::AcqRel);
        let meta = DeviceMeta {
            dev_id: DevId {
                major: DeviceMajor::Block,
                minor: id,
            },
            name: format!("sda{}", id),
            need_mapping: false,
            mmio_ranges: 
                pci_dev.ranges
                    .iter()
                    .map(|r| r.start as usize..r.end as usize)
                    .collect(),
            irq_no: None,
            dtype: crate::devices::DeviceType::Block,
        };
        Self { blk, meta }
    }
}

