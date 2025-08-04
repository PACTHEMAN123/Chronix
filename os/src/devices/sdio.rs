use alloc::{sync::Arc, vec::Vec};
use fdt::Fdt;

use crate::{devices::mmio::MmioDeviceDescripter, drivers::block::MMCBlock};



pub fn scan_sdio_blk(root: &Fdt) -> Option<Arc<MMCBlock>> {
    for node in root.find_all_nodes("/soc/mmc") {
        if node.reg().is_none() {
            continue;
        }
        let base_addr = node.reg().unwrap().next().unwrap().starting_address as usize;
        let size = node.reg().unwrap().next().unwrap().size.unwrap();
        log::info!("SD card host controller found at {:#x}", base_addr);
        let fd = MmioDeviceDescripter {
            mmio_region: base_addr..base_addr+size
        };
        let mmc_blk = MMCBlock::new(fd);
        return Some(Arc::new(mmc_blk))
    }
    None
}