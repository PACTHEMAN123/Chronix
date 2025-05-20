//! Device Manager 

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use fdt::Fdt;
use hal::{constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::MapPerm};

use crate::{drivers::serial::UART0, mm::{vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal}, KVMSPACE}, processor::processor::PROCESSORS};

use super::{block::{scan_mmio_blk_device, scan_pci_blk_device}, plic::{scan_plic_device, PLIC}, serial::scan_char_device, DevId, Device, DeviceMajor};

type IrqNo = usize;

/// Chronix's device manager
/// responsible for:
/// Creates device instance from device tree,
/// Maintains device instances lifetimes
/// Mapping interrupt No to device
pub struct DeviceManager {    
    /// Optional PLIC (Platform-Level Interrupt Controller) to manage external
    /// interrupts.
    pub plic: Option<PLIC>,
    /// mapping from device id to device instance
    pub devices: BTreeMap<DevId, Arc<dyn Device>>,
    /// mapping from irq no to device instance
    pub irq_map: BTreeMap<IrqNo, Arc<dyn Device>>
}

impl DeviceManager {
    /// create a new device manager
    pub fn new() -> Self {
        Self {
            plic: None,
            devices: BTreeMap::new(),
            irq_map: BTreeMap::new(),
        }
    }

    
    fn plic(&self) -> &PLIC {
        self.plic.as_ref().unwrap()
    }

    /// Device Init Stage1: scan the whole device tree and create instances
    /// map DevId to device, map IrqNo to device
    pub fn map_devices(&mut self, device_tree: &Fdt) {
        // map char device
        let serial = scan_char_device(device_tree);
        self.devices.insert(serial.dev_id(), serial.clone());
        self.irq_map.insert(serial.irq_no().unwrap(), serial.clone());
        
        // map block device
        // now not support for blk interrupt
        #[cfg(target_arch="loongarch64")]
        {
            let virtio_pci_blk = scan_pci_blk_device(device_tree);
            if let Some(blk) = virtio_pci_blk {
                self.devices.insert(blk.dev_id(), blk.clone());
            };
        }
        

        let virtio_mmio_blk = scan_mmio_blk_device(device_tree);
        if let Some(blk) = virtio_mmio_blk {
            self.devices.insert(blk.dev_id(), blk.clone());
        };


        let plic = scan_plic_device(device_tree);
        if let Some(plic) = plic {
            self.plic = Some(plic);
        }
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
            if dev.meta().need_mapping == false {
                continue;
            }
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
        // map plic
        if let Some(plic) = &self.plic {
            let paddr_start = plic.mmio_base;
            let vaddr_start = paddr_start | Constant::KERNEL_ADDR_SPACE.start;
            let size = plic.mmio_size;
            log::info!("[Device Manager]: mapping PLIC, from phys addr {:#x} to virt addr {:#x}, size {:#x}", paddr_start, vaddr_start, size);
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

    /// enable interrupt for device
    pub fn enable_irq(&mut self) {
        #[cfg(feature="smp")]
        for i in 0..MAX_PROCESSORS * 2 {
            for dev in self.devices.values() {
                if let Some(irq) = dev.irq_no() {
                    self.plic().enable_irq(irq, i);
                    log::info!("Enable external interrupt:{irq}, context:{i}");
                }
            }
        }
        #[cfg(not(feature="smp"))]
        for i in 0..2 {
            for dev in self.devices.values() {
                if let Some(irq) = dev.irq_no() {
                    self.plic().enable_irq(irq, i);
                    log::info!("Enable external interrupt:{irq}, context:{i}");
                }
            }
        }
        unsafe {
            Instruction::enable_external_interrupt();
        }
    }
    /// handle interrupt
    pub fn handle_irq(&self) {
        fn irq_ctx() -> usize {
            #[cfg(not(feature="smp"))]
            {
                1
            }
            #[cfg(feature="smp")]
            {
                todo!()
            }
        }
        unsafe { Instruction::disable_interrupt() };
        log::trace!("[Device Manager]: handle interrupt");
        if let Some(irq_num) = self.plic().claim_irq(irq_ctx()) {
            if let Some(dev) = self.irq_map.get(&irq_num) {
                dev.handle_irq();
                self.plic().complete_irq(irq_num, irq_ctx());
                return;
            }
        } 
    }
}