use alloc::{sync::Arc, vec::Vec};
use fdt::Fdt;

use crate::{devices::mmio::MmioDeviceDescripter, drivers::block::MMCBlock};



pub fn probe_sdio_blk(root: &Fdt) -> Vec<Arc<MMCBlock>> {
    let mut mmc_blks = Vec::new();
    for node in root.find_all_nodes("/soc/sdio") {
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
        mmc_blks.push(Arc::new(mmc_blk));
    }
    mmc_blks
}