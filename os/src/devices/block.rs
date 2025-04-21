//! scan the device tree to get a block device

use alloc::{string::ToString, sync::Arc};
use fatfs::info;
use fdt::Fdt;
use hal::constant::{Constant, ConstantsHal};

use crate::drivers::block::{VirtIOMMIOBlock, VirtIOPCIBlock};

pub fn scan_pci_blk_device(device_tree: &Fdt) -> Option<Arc<VirtIOPCIBlock>> {
    // find the pci line
    let pci = device_tree
        .find_compatible(&[
            "pci-host-ecam-generic",
        ]);
    if pci.is_none() {
        return None;
    }
    let pci = pci.unwrap();
    let reg = pci.reg().unwrap().next().unwrap();
    let base_paddr = reg.starting_address as usize;
    let size = reg.size.unwrap() as usize;
    let base_vaddr = base_paddr | Constant::KERNEL_ADDR_SPACE.start;
    log::info!("[Device Tree]: found PCI range at [{:#x}, {:#x}]", base_paddr, base_paddr + size);
    log::info!("[Device Tree]: mapping {:#x} to {:#x}", base_paddr, base_vaddr);

    // try to find the PCI memory range
    // 0x200_0000 indicates memory
    let ranges = pci.ranges().expect("[Device Tree]: ranges not found");
    let memory_range = ranges
        .into_iter()
        .filter(|r| r.child_bus_address_hi == 0x2000000)
        .next();

    if memory_range.is_none() {
        return None;
    }
    let memory_range = memory_range.unwrap();
    log::info!("find range: {:#x} {:#x} {:#x} {:#x}", memory_range.child_bus_address_hi, memory_range.child_bus_address, memory_range.parent_bus_address, memory_range.size);
    
    Some(Arc::new(
        VirtIOPCIBlock::new(
        memory_range.child_bus_address, 
        memory_range.size, 
        base_paddr,
    )))
}

pub fn scan_mmio_blk_device(device_tree: &Fdt) -> Option<Arc<VirtIOMMIOBlock>> {
    // UGLY: now we use specific device
    let node = device_tree.find_node("/soc/virtio_mmio@10001000");
    if node.is_none() {
        return None;
    }
    let node = node.unwrap();
    let reg = node.reg().unwrap().next().unwrap();
    let mmio_paddr_start = reg.starting_address as usize;
    let mmio_size = reg.size.unwrap();
    Some(Arc::new(VirtIOMMIOBlock::new(mmio_paddr_start, mmio_size)))
}