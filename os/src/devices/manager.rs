//! Device Manager 

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use fdt::Fdt;
use hal::{constant::{Constant, ConstantsHal}, pagetable::MapPerm};

use crate::{drivers::serial::UART0, mm::{vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal}, KVMSPACE}};

use super::{serial::scan_char_device, DevId, Device, DeviceMajor};

type IrqNo = usize;

/// Chronix's device manager
/// responsible for:
/// Creates device instance from device tree,
/// Maintains device instances lifetimes
/// Mapping interrupt No to device
pub struct DeviceManager {
    /// mapping from device id to device instance
    pub devices: BTreeMap<DevId, Arc<dyn Device>>,
    /// mapping from irq no to device instance
    pub irq_map: BTreeMap<IrqNo, Arc<dyn Device>>
}

impl DeviceManager {
    /// create a new device manager
    pub fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
            irq_map: BTreeMap::new(),
        }
    }

    /// Device Init Stage1: scan the whole device tree and create instances
    /// map DevId to device, map IrqNo to device
    pub fn map_devices(&mut self, device_tree: &Fdt) {
        // map char device
        let serial = scan_char_device(device_tree);
        self.devices.insert(serial.dev_id(), serial.clone());
        self.irq_map.insert(serial.irq_no().unwrap(), serial.clone());
        
        // map block device
        // TODO
    }

    /// Device Init Stage2: map the mmio region
    /// WARNING: this method can only be called after:
    /// 1. finish kernel page table initialize
    /// 2. extract device tree and get all the devices
    pub fn map_mmio_area(&self) {
        for (_, dev) in &self.devices {
            let paddr_start = dev.mmio_base();
            let vaddr_start = paddr_start | Constant::KERNEL_ADDR_SPACE.start;
            let size = dev.mmio_size();
            log::info!("[Device Manager]: mapping {}, from phys addr {:#x} to virt addr {:#x}, size {:#x}", dev.name(), paddr_start, vaddr_start, size);
            KVMSPACE.lock().push_area(
                KernVmArea::new(
                    vaddr_start.into()..(vaddr_start + size).into(),
                    KernVmAreaType::MemMappedReg, 
                    MapPerm::R | MapPerm::W,
                ),
                None
            );
        }
    }

    /// Device Init Stage3: init all devices
    pub fn init_devices(&self) {
        for (_, dev) in &self.devices {
            log::info!("[Device Manager]: init device: {}", dev.name());
            dev.init();
        }
    }

    /// using given device major to find devices
    /// return a vector of devices belongs to this major
    pub fn find_dev_by_major(&self, major: DeviceMajor) -> Vec<Arc<dyn Device>>{
        self.devices
            .iter()
            .filter(|(dev_id, _)| dev_id.major == major)
            .map(|(_, dev)| dev)
            .cloned()
            .collect()
    }
}