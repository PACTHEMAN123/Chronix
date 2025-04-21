//! VirtIO Block using PCI transport
//! 

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use hal::constant::{Constant, ConstantsHal};
use lazy_static::lazy_static;
use virtio_drivers::transport::pci::bus::{BarInfo, Cam, Command, MemoryBarType, MmioCam, PciRoot};

use crate::config::BLOCK_SIZE;
use crate::devices::{BlockDevice, DevId, Device, DeviceMajor, DeviceMeta};
use crate::drivers::dma::VirtioHal;
use crate::mm::FrameTracker;
use crate::sync::UPSafeCell;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::pci::PciTransport;
use virtio_drivers::transport::{DeviceType, Transport};
use virtio_drivers::BufferDirection;

pub struct VirtIOPCIBlock {
    meta: DeviceMeta,
    blk: UPSafeCell<VirtIOBlk<VirtioHal, PciTransport>>,
}

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = UPSafeCell::new(Vec::new());
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
    pub fn new(start: usize, size: usize, pci_paddr: usize) -> Self {
        let pci_vaddr = pci_paddr | Constant::KERNEL_ADDR_SPACE.start;
        let mut allocator = PciMemory32Allocator::for_pci_ranges(start as _, (start + size) as _);
        let mut root = PciRoot::new(unsafe { 
            MmioCam::new(pci_vaddr as *mut u8, Cam::Ecam)
        });
        let mut device_function = None;
        for (df, dfi) in root.enumerate_bus(0) {
            if dfi.class == 1 {
                device_function = Some(df);
                break;
            }
        }
        let device_function = device_function.expect("block device not found");
        for (i, info) in root.bars(device_function).unwrap().into_iter().enumerate() {
            let Some(info) = info else { continue };
            log::info!("BAR {}: {}", i, info);
            if let BarInfo::Memory {
                address_type, size, ..
            } = info {
                match address_type {
                    MemoryBarType::Width32 => {
                        if size > 0 {
                            let addr = allocator.allocate_memory_32(size);
                            log::info!("Allocated address: {:#x}", addr);
                            root.set_bar_32(device_function, i as u8, addr as u32);
                        }
                    },
                    MemoryBarType::Width64 => {
                        if size > 0 {
                            let addr = allocator.allocate_memory_32(size);
                            log::info!("Allocated address: {:#x}", addr);
                            root.set_bar_64(device_function, i as u8, addr as u64);
                        }
                    },
                    _ => panic!("Memory BAR address type {:?} not supported.", address_type),
                }
            }
            
        }
        root.set_command(
            device_function,
            Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
        );
        let (status, command) = root.get_status_command(device_function);
        log::info!(
            "Allocated BARs and enabled device, status {:?} command {:?}",
            status, command
        );
        // dump_bar_contents(&mut root, device_function, 4);
        let mut transport = PciTransport::new::<VirtioHal, _>(&mut root, device_function).unwrap();
        log::info!(
            "Detected virtio PCI device with device type {:?}, features {:#018x}",
            transport.device_type(),
            transport.read_device_features(),
        );
        let blk = UPSafeCell::new(
            VirtIOBlk::<VirtioHal, PciTransport>::new(transport).expect("failed to create blk driver"),
        );
        let meta = DeviceMeta {
            dev_id: DevId {
                major: DeviceMajor::Block,
                minor: 0,
            },
            name: "virtio-pci-blk".to_string(),
            need_mapping: false, // WARN: we assume only la will use PCI block
            mmio_base: start, // WARN: not sure about the mmio
            mmio_size: size,
            irq_no: None, // TODO: support interrupt for block device
            dtype: crate::devices::DeviceType::Block,
        };
        Self { blk, meta }
    }
}

/// Allocates 32-bit memory addresses for PCI BARs.
struct PciMemory32Allocator {
    start: u32,
    end: u32,
}

impl PciMemory32Allocator {
    /// Creates a new allocator based on the ranges property of the given PCI node.
    pub fn for_pci_ranges(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
        }
    }

    /// Allocates a 32-bit memory address region for a PCI BAR of the given power-of-2 size.
    ///
    /// It will have alignment matching the size. The size must be a power of 2.
    pub fn allocate_memory_32(&mut self, size: u32) -> u32 {
        assert!(size.is_power_of_two());
        let allocated_address = align_up(self.start, size);
        assert!(allocated_address + size <= self.end);
        self.start = allocated_address + size;
        allocated_address
    }
}

const fn align_up(value: u32, alignment: u32) -> u32 {
    ((value - 1) | (alignment - 1)) + 1
}







