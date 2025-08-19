//! Device Manager 

use alloc::{collections::btree_map::BTreeMap, string::ToString, sync::Arc, vec::Vec,vec};
use fdt::Fdt;
use hal::{addr::PhysAddr, board::MAX_PROCESSORS, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, irq::{IrqCtrl, IrqCtrlHal}, pagetable::MapPerm, println};
use virtio_drivers::transport::{mmio::{MmioTransport, VirtIOHeader}, Transport};

use crate::{devices::DeviceMeta, drivers::{block::{loongarch::ahci_blk::AchiBlock, VirtIOMMIOBlock, VirtIOPCIBlock}, net::{loopback::LoopbackDevice, virtio_net::VirtIoNetDevImpl}, serial::UART0}, fs::procfs::interrupt::IRQ_COUNTER, mm::{vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal}, MmioMapper, KVMSPACE}, net::init_network, processor::processor::PROCESSORS};

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
    pub irq_map: BTreeMap<IrqNo, Arc<dyn Device>>,
    /// net device meta
    pub net_meta: Option<DeviceMeta>,
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
            net_meta: None,
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

        log::info!("Device: {}", device_tree.root().model());

        // map char device
        let serial = scan_char_device(device_tree);
        self.devices.insert(serial.dev_id(), serial.clone());
        self.irq_map.insert(serial.irq_no().unwrap(), serial.clone());
        let ahci = Arc::new(AchiBlock::new());
        self.devices.insert(ahci.dev_id(), ahci.clone());

        if let Some(irq_ctrl) = IrqCtrl::from_dt(device_tree, MmioMapper) {
            self.irq_ctrl = Some(irq_ctrl);
        }
        
        // if let Some(mut pci) = PciManager::scan_pcie_root(device_tree) {
        //     for mut device in pci.enumerate_devices() {
        //         // pci bus has an advantage: no need to map or allocate 
        //         // memory to recognize the device type.
        //         let dev_class: PciDeviceClass = device.func_info.class.into();
        //         let dev = match dev_class {
        //             PciDeviceClass::MassStorageContorller => {
        //                 pci.init_device(&mut device).unwrap();
        //                 Arc::new(VirtIOPCIBlock::new(device))
        //             }
        //             _ => continue
        //         };

        //         if let Some(irq_no) = dev.irq_no() {
        //             self.irq_map.insert(irq_no, dev.clone());
        //         }
        //         self.devices.insert(dev.dev_id(), dev);
        //     }
        //     self.pci = Some(pci);
        // }
        
        // let mmio = MmioManager::scan_mmio_root(device_tree);
        // for deivce in mmio.enumerate_devices() {
        //     if let Ok(mmio_transport) = deivce.transport() {

        //         let dev = match mmio_transport.device_type() {
        //             virtio_drivers::transport::DeviceType::Block => {
        //                 Arc::new(VirtIOMMIOBlock::new(deivce.clone(), mmio_transport))
        //             }
        //             _ => continue
        //         };

        //         if let Some(irq_no) = dev.irq_no() {
        //             self.irq_map.insert(irq_no, dev.clone());
        //         }
        //         self.devices.insert(dev.dev_id(), dev);
        //     }
        // }
        // for device in mmio.enumerate_devices() {
        //     let mmio_ranges = &device.mmio_region;
        //     if let Ok(mmio_transport) = device.transport() {
        //         match mmio_transport.device_type() {
        //             virtio_drivers::transport::DeviceType::Network => {
        //                 log::warn!("find a virtio-net device, use it");
        //                 self.net_meta = Some(
        //                     DeviceMeta {
        //                         dev_id: DevId {
        //                             major: DeviceMajor::Net,
        //                             minor: 0,
        //                         },
        //                         name: "virtio-blk".to_string(),
        //                         need_mapping: false,
        //                         mmio_ranges: vec!(mmio_ranges.clone()),
        //                         irq_no: None,
        //                         dtype: crate::devices::DeviceType::Net,  
        //                     }
        //                 )
        //             }
        //             _ => continue,
        //         }
        //         if self.net_meta.is_some() {
        //             break;
        //         }
        //     }
        // }
        // self.mmio = Some(mmio);

        if let Some(sdio_blk) = scan_sdio_blk(device_tree) {
            log::info!("find a sdio block device");
            self.devices.insert(sdio_blk.dev_id(), sdio_blk);
        }

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
        // #[cfg(target_arch = "riscv64")]
        // if let Some(irq_ctrl) = &self.irq_ctrl{
        //     let plic = &irq_ctrl.plic;
        //     let paddr_start = plic.mmio_base;
        //     let vaddr_start = paddr_start | Constant::KERNEL_ADDR_SPACE.start;
        //     let size = plic.mmio_size;
        //     log::info!("[Device Manager]: mapping PLIC, from phys addr {:#x} to virt addr {:#x}, size {:#x}", paddr_start, vaddr_start, size);
        //     KVMSPACE.lock().push_area(
        //         KernVmArea::new(
        //             vaddr_start.into()..(vaddr_start + size).into(),
        //             KernVmAreaType::MemMappedReg, 
        //             MapPerm::R | MapPerm::W,
        //         ),
        //         None
        //     );
        // }

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

    /// using given device name and major to find devices
    pub fn find_dev_by_name(&self, name: &str, major: DeviceMajor) -> Arc<dyn Device> {
        self.devices
            .iter()
            .find(|(dev_id, dev)| 
            dev_id.major == major && dev.meta().name == name)
            .map(|(_, dev)| dev.clone())
            .expect("device not found")
    }

    /// enable interrupt for device
    pub fn enable_irq(&mut self) {
        // #[cfg(feature="smp")]
        // use hal::board::MAX_PROCESSORS;
        // #[cfg(feature="smp")]
        // // todo!
        // for i in 0..MAX_PROCESSORS * 2 {
        //     for dev in self.devices.values() {
        //         if let Some(irq) = dev.irq_no() {
        //             self.irq_ctrl().enable_irq(irq);
        //             log::info!("Enable external interrupt:{irq}, context:{i}");
        //         }
        //     }
        // }
        // #[cfg(not(feature="smp"))]
        // for i in 0..2 {
        //     for dev in self.devices.values() {
        //         if let Some(irq) = dev.irq_no() {
        //             self.irq_ctrl().enable_irq(irq, i);
        //             log::info!("Enable external interrupt:{irq}, context:{i}");
        //         }
        //     }
        // }
        for dev in self.devices.values() {
            if let Some(irq) = dev.irq_no() {
                self.irq_ctrl().enable_irq(irq, 0);
                log::info!("Enable external interrupt:{irq}, context:{}", 0);
            }
        }
        unsafe {
            Instruction::enable_external_interrupt();
        }
    }
    /// handle interrupt
    pub fn handle_irq(&self) {
        unsafe { Instruction::disable_interrupt() };
        if let Some(irq_num) = self.irq_ctrl().claim_irq(self.irq_ctx()) {
            IRQ_COUNTER.lock().add_irq(irq_num);
            // log::warn!("[Device manager] get irq no {irq_num}");
            if let Some(dev) = self.irq_map.get(&irq_num) {
                dev.handle_irq();
                self.irq_ctrl().complete_irq(irq_num, self.irq_ctx());
                return;
            }
        } 
    }

    pub fn init_net(&self) {
        if let Some(meta) = self.net_meta.as_ref() {
            let addr_range = &meta.mmio_ranges[0];
            let paddr = addr_range.start;
            let vaddr = paddr | Constant::KERNEL_ADDR_SPACE.start;
            let size = addr_range.clone().count();
            let header = core::ptr::NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
            let transport = unsafe {
                MmioTransport::new(header, size)
            }.unwrap();
            let _dev = VirtIoNetDevImpl::new(transport).unwrap();
            log::warn!("use virtio-net device");
            // init_network(_dev, true);
            init_network(LoopbackDevice::new(),false);
        }else  {
            log::warn!("use loopback device");
            init_network(LoopbackDevice::new(),false);
        }
    }

    // get the current irq context id based on hart id
    pub fn irq_ctx(&self) -> usize {
        current_processor_id() * 2
    }
}
