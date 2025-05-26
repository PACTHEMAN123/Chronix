//! VirtIO Block using MMIO transport

use core::sync::atomic::Ordering;

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::{format, vec};
use hal::constant::{Constant, ConstantsHal};
use hal::pagetable::MapPerm;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::{self, Transport};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use crate::config::BLOCK_SIZE;
use crate::devices::mmio::MmioDeviceDescripter;
use crate::devices::{BlockDevice, DevId, Device, DeviceMajor};
use crate::drivers::dma::VirtioHal;

use crate::mm::vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal};
use crate::mm::KVMSPACE;
use crate::{devices::DeviceMeta, sync::UPSafeCell};

use super::BLK_ID;

pub struct VirtIOMMIOBlock {
    blk: UPSafeCell<VirtIOBlk<VirtioHal, MmioTransport>>,
    meta: DeviceMeta,
}

impl BlockDevice for VirtIOMMIOBlock {

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

impl Device for VirtIOMMIOBlock {
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

impl VirtIOMMIOBlock {
    // use a VirtIO MMIO paddr
    pub fn new(mmio_dev: MmioDeviceDescripter, mmio_transport: MmioTransport) -> Self {
        let blk = UPSafeCell::new(
            VirtIOBlk::<VirtioHal, MmioTransport>::new(mmio_transport).expect("failed to create blk driver"),
        );
        let id = BLK_ID.fetch_add(1, Ordering::AcqRel);
        let meta = DeviceMeta {
            dev_id: DevId {
                major: DeviceMajor::Block,
                minor: id,
            },
            name: format!("sda{}", id),
            need_mapping: false,
            mmio_ranges: vec![mmio_dev.mmio_region],
            irq_no: None,
            dtype: crate::devices::DeviceType::Block,
        };
        Self { blk, meta }
    }
}