use core::{ops::Range, ptr::null_mut};

use alloc::{sync::Arc, vec::Vec};
use fdt::{node::FdtNode, Fdt};
use hal::{addr::VirtAddr, constant::{Constant, ConstantsHal}, pagetable::MapPerm};
use spin::Lazy;
use virtio_drivers::transport::{pci::{bus::{BarInfo, BusDeviceIterator, Cam, Command, DeviceFunction, DeviceFunctionInfo, MemoryBarType, MmioCam, PciRoot}, PciTransport}, Transport};

use crate::{drivers::dma::VirtioHal, mm::{vm::{KernVmArea, KernVmAreaType, KernVmSpaceHal}, KVMSPACE}, sync::{lazy::SyncLazyCell, mutex::SpinNoIrqLock}};

use super::Device;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PciDeviceClass {
    UnclassifiedDevice = 0x0,
    MassStorageContorller = 0x1,
    NetWorkContorller = 0x2,
    DisplayContorller = 0x3,
    MultimediaController = 0x4,
    MemoryController = 0x5,
    Bridge = 0x6,
    CommunicationController = 0x7,
    GenericSystemPeripheral = 0x8,
    InputDeviceController = 0x9,
    DockingStation = 0xa,
    Processor = 0xb,
    SerialBusController = 0xc,
    WirelessController = 0xd,
    IntelligentController = 0xe,
    SatelliteCommunicationsController = 0xf,
    EncryptionController = 0x10,
    SignalProcessingController = 0x11,
    ProcessingAccelerators = 0x12,
    NonEssentialInstrumentation = 0x13,
    Coprocessor = 0x40,
    UnassignedClass = 0xff,
}

impl From<u8> for PciDeviceClass {
    fn from(value: u8) -> Self {
        match value {
            0x00 => PciDeviceClass::UnclassifiedDevice,
            0x01 => PciDeviceClass::MassStorageContorller,
            0x02 => PciDeviceClass::NetWorkContorller,
            0x03 => PciDeviceClass::DisplayContorller,
            0x04 => PciDeviceClass::MultimediaController,
            0x05 => PciDeviceClass::MemoryController,
            0x06 => PciDeviceClass::Bridge,
            0x07 => PciDeviceClass::CommunicationController,
            0x08 => PciDeviceClass::GenericSystemPeripheral,
            0x09 => PciDeviceClass::InputDeviceController,
            0x0a => PciDeviceClass::DockingStation,
            0x0b => PciDeviceClass::Processor,
            0x0c => PciDeviceClass::SerialBusController,
            0x0d => PciDeviceClass::WirelessController,
            0x0e => PciDeviceClass::IntelligentController,
            0x0f => PciDeviceClass::SatelliteCommunicationsController,
            0x10 => PciDeviceClass::EncryptionController,
            0x11 => PciDeviceClass::SignalProcessingController,
            0x12 => PciDeviceClass::ProcessingAccelerators,
            0x13 => PciDeviceClass::NonEssentialInstrumentation,
            0x40 => PciDeviceClass::Coprocessor,
            0xff => PciDeviceClass::UnassignedClass,
            _ => PciDeviceClass::UnassignedClass,
        }
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PciRangeType {
    ConfigurationSpace,
    IoSpace,
    Memory32,
    Memory64,
}

impl From<u32> for PciRangeType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::ConfigurationSpace,
            1 => Self::IoSpace,
            2 => Self::Memory32,
            3 => Self::Memory64,
            _ => panic!("Tried to convert invalid range type {}", value),
        }
    }
}


/// Allocates 32-bit memory addresses for PCI BARs.
pub struct PciMemory32Allocator {
    start: u32,
    end: u32,
}

impl PciMemory32Allocator {
    /// Creates a new allocator based on the ranges property of the given PCI node.
    pub fn for_pci_ranges(pci_node: &FdtNode) -> Self {
        let mut memory_32_address = 0;
        let mut memory_32_size = 0;
        for range in pci_node.ranges().expect("[PciMemory32Allocator]: ranges not found") {
            let prefetchable = range.child_bus_address_hi & 0x4000_0000 != 0;
            let range_type = PciRangeType::from((range.child_bus_address_hi & 0x0300_0000) >> 24);
            let bus_address = range.child_bus_address as u64;
            let cpu_physical = range.parent_bus_address as u64;
            let size = range.size as u64;
            log::info!(
                "range: {:?} {}prefetchable bus address: {:#018x} host physical address: {:#018x} size: {:#018x}",
                range_type,
                if prefetchable { "" } else { "non-" },
                bus_address,
                cpu_physical,
                size,
            );
            // Use the largest range within the 32-bit address space for 32-bit memory, even if it
            // is marked as a 64-bit range. This is necessary because crosvm doesn't currently
            // provide any 32-bit ranges.
            if !prefetchable
                && matches!(range_type, PciRangeType::Memory32 | PciRangeType::Memory64)
                && size > memory_32_size.into()
                && bus_address + size < u32::MAX.into()
            {
                assert_eq!(bus_address, cpu_physical);
                memory_32_address = u32::try_from(cpu_physical).unwrap();
                memory_32_size = u32::try_from(size).unwrap();
            }
        }
        if memory_32_size == 0 {
            panic!("No 32-bit PCI memory region found.");
        }
        let vaddr: VirtAddr = (memory_32_address as usize | Constant::KERNEL_ADDR_SPACE.start).into();
        KVMSPACE.lock().push_area(
            KernVmArea::new(
                vaddr..vaddr + memory_32_size as usize, 
                KernVmAreaType::MemMappedReg, 
                MapPerm::R | MapPerm::W
            ),
            None
        );
        Self {
            start: memory_32_address,
            end: memory_32_address + memory_32_size,
        }
    }

    /// Allocates a 32-bit memory address region for a PCI BAR of the given power-of-2 size.
    ///
    /// It will have alignment matching the size. The size must be a power of 2.
    pub fn allocate_memory_32(&mut self, size: u32) -> u32 {
        let size = next_power_of_two(size);
        let allocated_address = align_up(self.start, size);
        assert!(allocated_address + size <= self.end);
        self.start = allocated_address + size;
        if self.start > self.end {
            log::warn!("[PciMemAllocator] out of memory, device may not work properly");
        }
        allocated_address as u32
    }
}

// IRQ Model of Loongarch in Qemu
// more details: https://docs.kernel.org/6.2/loongarch/irq-chip-model.html
//     +-----+     +---------+     +-------+
//     | IPI | --> | CPUINTC | <-- | Timer |
//     +-----+     +---------+     +-------+
//                      ^
//                      |
//                 +---------+
//                 | EIOINTC |
//                 +---------+
//                  ^       ^
//                  |       |
//           +---------+ +---------+
//           | PCH-PIC | | PCH-MSI |
//           +---------+ +---------+
//                ^           ^
//                |           |
//           +---------+ +---------+
//           | Devices | | Devices |
//           +---------+ +---------+

pub struct PciManager {
    base_addr: usize,
    pub mem_alloc: PciMemory32Allocator,
    pub root: PciRoot<MmioCam>,
}

pub struct PciDeviceDescriptor {
    pub func: DeviceFunction,
    pub func_info: DeviceFunctionInfo,
    pub transport: Option<PciTransport>,
    pub ranges: Vec<Range<u32>>
}

pub struct PciDeviceDescIter {
    bus_iter: BusDeviceIterator<MmioCam>
}

impl Iterator for PciDeviceDescIter {
    type Item = PciDeviceDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let (func, func_info) = self.bus_iter.next()?;
        Some(
            PciDeviceDescriptor { 
                func, func_info, 
                transport: None, 
                ranges: Vec::new()
            }
        )
    }
}

impl PciManager {
    pub fn scan_pcie_root(root: &Fdt) -> Option<PciManager> {
        if let Some(pcie_node) = root.find_compatible(&["pci-host-ecam-generic"]) {
            let reg = pcie_node.reg().unwrap().next().unwrap();
            let base_paddr = reg.starting_address as usize;
            let size = reg.size.unwrap() as usize;
            let base_vaddr = base_paddr | Constant::KERNEL_ADDR_SPACE.start;
            log::info!("[Device Tree]: found PCI range at [{:#x}, {:#x}]", base_paddr, base_paddr + size);
            log::info!("[Device Tree]: mapping {:#x} to {:#x}", base_paddr, base_vaddr);
            KVMSPACE.lock().push_area(KernVmArea::new(
                base_vaddr.into()..(base_vaddr+size).into(),
                KernVmAreaType::MemMappedReg,
                MapPerm::R | MapPerm::W
            ), 
            None);
            let mem_alloc = PciMemory32Allocator::for_pci_ranges(&pcie_node);
            let mmio_cam = unsafe { MmioCam::new(base_vaddr as *mut u8, Cam::Ecam) };
            Some(Self{
                base_addr: base_vaddr,
                mem_alloc: mem_alloc,
                root: PciRoot::new(mmio_cam),
            })
        } else {
            None
        }
    }

    pub fn enumerate_devices(&self) -> PciDeviceDescIter {
        PciDeviceDescIter {
            bus_iter: self.root.enumerate_bus(0)
        }
    }

    /// allocate device's bar memory, create transport
    pub fn init_device(&mut self, device: &mut PciDeviceDescriptor) -> Result<(), ()> {
        let PciDeviceDescriptor { func, ..} = *device;
        for (i, info) in self.root.bars(func).unwrap().into_iter().enumerate() {
            let Some(info) = info else { continue };
            log::info!("BAR {}: {}", i, info);
            if let BarInfo::Memory {
                address_type, size, ..
            } = info {
                match address_type {
                    MemoryBarType::Width32 => {
                        if size > 0 {
                            let addr = self.mem_alloc.allocate_memory_32(size);
                            log::info!("Allocated address: {:#x}", addr);
                            self.root.set_bar_32(func, i as u8, addr as u32);
                            device.ranges.push(addr..addr+size);
                        }
                    },
                    MemoryBarType::Width64 => {
                        if size > 0 {
                            let addr = self.mem_alloc.allocate_memory_32(size);
                            log::info!("Allocated address: {:#x}", addr);
                            self.root.set_bar_64(func, i as u8, addr as u64);
                            device.ranges.push(addr..addr+size);
                        }
                    },
                    _ => {
                        log::warn!("Memory BAR address type {:?} not supported.", address_type);
                        return Err(())
                    }
                }
            }
        }
        self.root.set_command(
            func,
            Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
        );
        let (status, command) = self.root.get_status_command(func);
        log::info!(
            "Allocated BARs and enabled device, status {:?} command {:?}",
            status, command
        );
        let mut transport = 
            PciTransport::new::<VirtioHal, _>(&mut self.root, func).map_err(|_| ())?;
        log::info!(
            "Detected virtio PCI device with device type {:?}, features {:#018x}",
            transport.device_type(),
            transport.read_device_features(),
        );
        device.transport = Some(transport);
        Ok(())
    }
}

const fn align_up(value: u32, alignment: u32) -> u32 {
    ((value - 1) | (alignment - 1)) + 1
}

pub const fn next_power_of_two(mut x: u32) -> u32 {
    if x == 0 {
        return 1
    }
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    return x+1;
}