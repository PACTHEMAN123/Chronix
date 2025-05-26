//! Device Manager 

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use fdt::Fdt;
use hal::{constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, irq::{IrqCtrl, IrqCtrlHal}, pagetable::MapPerm, println};
use virtio_drivers::transport::Transport;

use crate::{drivers::{block::{VirtIOMMIOBlock, VirtIOPCIBlock}, serial::UART0}, mm::{vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal}, MmioMapper, KVMSPACE}, processor::processor::PROCESSORS};

use super::{mmio::MmioManager, pci::{PciDeviceClass, PciManager}, plic::{scan_plic_device, PLIC}, serial::scan_char_device, DevId, Device, DeviceMajor};

type IrqNo = usize;

/// Chronix's device manager
/// responsible for:
/// Creates device instance from device tree,
/// Maintains device instances lifetimes
/// Mapping interrupt No to device
pub struct DeviceManager {    
    /// Optional interrupt controller
    pub irq_ctrl: Option<IrqCtrl>,
    /// Optional PCI
    pub pci: Option<PciManager>,
    /// Optional MMIO
    pub mmio: Option<MmioManager>,
    /// mapping from device id to device instance
    pub devices: BTreeMap<DevId, Arc<dyn Device>>,
    /// mapping from irq no to device instance
    pub irq_map: BTreeMap<IrqNo, Arc<dyn Device>>
}

impl DeviceManager {
    /// create a new device manager
    pub fn new() -> Self {
        Self {
            irq_ctrl: None,
            pci: None,
            mmio: None,
            devices: BTreeMap::new(),
            irq_map: BTreeMap::new(),
        }
    }

    
    fn irq_ctrl(&self) -> &IrqCtrl {
        self.irq_ctrl.as_ref().unwrap()
    }

    fn pci(&self) -> &PciManager {
        self.pci.as_ref().unwrap()
    }

    fn mmio(&self) -> &MmioManager {
        self.mmio.as_ref().unwrap()
    }

    /// Device Init Stage1: scan the whole device tree and create instances
    /// map DevId to device, map IrqNo to device
    pub fn map_devices(&mut self, device_tree: &Fdt) {
        // map char device
        let serial = scan_char_device(device_tree);
        self.devices.insert(serial.dev_id(), serial.clone());
        self.irq_map.insert(serial.irq_no().unwrap(), serial.clone());

        if let Some(irq_ctrl) = IrqCtrl::from_dt(device_tree, MmioMapper) {
            self.irq_ctrl = Some(irq_ctrl);
        }
        
        if let Some(mut pci) = PciManager::scan_pcie_root(device_tree) {
            for mut device in pci.enumerate_devices() {
                // pci bus has an advantage: no need to map or allocate 
                // memory to recognize the device type.
                let dev_class: PciDeviceClass = device.func_info.class.into();
                let dev = match dev_class {
                    PciDeviceClass::MassStorageContorller => {
                        pci.init_device(&mut device).unwrap();
                        Arc::new(VirtIOPCIBlock::new(device))
                    }
                    _ => continue
                };

                if let Some(irq_no) = dev.irq_no() {
                    self.irq_map.insert(irq_no, dev.clone());
                }
                self.devices.insert(dev.dev_id(), dev);
            }
            self.pci = Some(pci);
        }
        
        let mmio = MmioManager::scan_mmio_root(device_tree);
        for deivce in mmio.enumerate_devices() {
            if let Ok(mmio_transport) = deivce.transport() {

                let dev = match mmio_transport.device_type() {
                    virtio_drivers::transport::DeviceType::Block => {
                        Arc::new(VirtIOMMIOBlock::new(deivce.clone(), mmio_transport))
                    }
                    _ => continue
                };

                if let Some(irq_no) = dev.irq_no() {
                    self.irq_map.insert(irq_no, dev.clone());
                }
                self.devices.insert(dev.dev_id(), dev);
            }
        }
        self.mmio = Some(mmio);

        // let plic = scan_plic_device(device_tree);
        // if let Some(plic) = plic {
        //     self.plic = Some(plic);
        // }
        // TODO
    }

    /// Device Init Stage2: map the mmio region
    /// WARNING: this method can only be called after:
    /// 1. finish kernel page table initialize
    /// 2. extract device tree and get all the devices
    pub fn map_mmio_area(&self) {
        for (_, dev) in &self.devices {
            for range in dev.mmio_ranges() {
                let paddr_start = range.start;
                let size = range.end - range.start;
                let vaddr_start = paddr_start | Constant::KERNEL_ADDR_SPACE.start;
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
                    self.irq_ctrl().enable_irq(irq);
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
        if let Some(irq_num) = self.irq_ctrl().claim_irq() {
            if let Some(dev) = self.irq_map.get(&irq_num) {
                dev.handle_irq();
                self.irq_ctrl().complete_irq(irq_num);
                return;
            }
        } 
    }
}