use core::ops::Range;
use alloc::vec::Vec;
use fdt::Fdt;
use hal::{constant::{Constant, ConstantsHal}, pagetable::MapPerm, println};
use virtio_drivers::transport::mmio::{MmioError, MmioTransport, VirtIOHeader};

use crate::mm::{vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal}, KVMSPACE};

#[derive(Clone)]
pub struct MmioDeviceDescripter {
    pub mmio_region: Range<usize>,
}

impl MmioDeviceDescripter {
    pub fn transport(&self) -> Result<MmioTransport, MmioError> {
        let paddr = self.mmio_region.start;
        let vaddr = paddr | Constant::KERNEL_ADDR_SPACE.start;
        let size = self.mmio_region.clone().count();
        let header = core::ptr::NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
        unsafe {
            MmioTransport::new(header, size)
        }
    }
}

pub struct MmioManager {
    pub devices: Vec<MmioDeviceDescripter>
}

impl MmioManager {
    pub fn scan_mmio_root(root: &Fdt) -> Self {
        let mut devices = Vec::new();
        for node in root.find_all_nodes("/soc/virtio_mmio") {
            if node.reg().is_none() { continue; }
            for region in node.reg().unwrap() {
                if let Some(size) = region.size {
                    let paddr = region.starting_address as usize;
                    let vaddr = paddr | Constant::KERNEL_ADDR_SPACE.start;
                    KVMSPACE.lock().push_area(
                        KernVmArea::new(
                            vaddr.into()..(vaddr + size).into(), 
                            KernVmAreaType::MemMappedReg, 
                            MapPerm::R | MapPerm::W,
                        ), 
                        None
                    );
                    
                    devices.push(MmioDeviceDescripter { 
                        mmio_region: paddr..paddr+size
                    });
                }
            }
        }
        MmioManager {
            devices
        }
    }

    pub fn enumerate_devices(&self) -> core::slice::Iter<'_, MmioDeviceDescripter> {
        self.devices.iter()
    }
}