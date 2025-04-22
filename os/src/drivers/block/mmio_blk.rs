//! VirtIO Block using MMIO transport

use alloc::string::ToString;
use alloc::sync::Arc;
use hal::constant::{Constant, ConstantsHal};
use hal::pagetable::MapPerm;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use crate::config::BLOCK_SIZE;
use crate::devices::{BlockDevice, DevId, Device, DeviceMajor};
use crate::drivers::dma::VirtioHal;

use crate::mm::vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal};
use crate::mm::KVMSPACE;
use crate::{devices::DeviceMeta, sync::UPSafeCell};

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
    pub fn new(paddr: usize, size: usize) -> Self {
        let vaddr = paddr | Constant::KERNEL_ADDR_SPACE.start;
        // UGLY: we need to read the header to construct
        // should map the MMIO area first
        KVMSPACE.lock().push_area(
            KernVmArea::new(
                vaddr.into()..(vaddr + size).into(), 
                KernVmAreaType::MemMappedReg, 
                MapPerm::R | MapPerm::W,
            ), 
            None
        );

        let header = core::ptr::NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
        let transport = unsafe {
            MmioTransport::new(header, size)
        }.unwrap();
        let blk = UPSafeCell::new(
            VirtIOBlk::<VirtioHal, MmioTransport>::new(transport).expect("failed to create blk driver"),
        );
        let meta = DeviceMeta {
            dev_id: DevId {
                major: DeviceMajor::Block,
                minor: 0,
            },
            name: "virtio-mmio-blk".to_string(),
            need_mapping: false, // WARN: we assume map before init the header
            mmio_base: vaddr, // WARN: not sure about the mmio
            mmio_size: size,
            irq_no: None, // TODO: support interrupt for block device
            dtype: crate::devices::DeviceType::Block,
        };
        Self { blk, meta }
    }
}