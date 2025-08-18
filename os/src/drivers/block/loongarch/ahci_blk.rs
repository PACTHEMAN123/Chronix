use core::{alloc::{GlobalAlloc, Layout}, sync::atomic::Ordering, time::Duration};

use alloc::{alloc::{Allocator, Global}, string::ToString, sync::Arc, vec::Vec, vec};
use hal::{constant::{Constant, ConstantsHal}, println};
use ls2k1000_ahci_driver::{drv_ahci::{ahci_init, ahci_sata_read_common, ahci_sata_write_common}, libahci::ahci_device, platform::{ahci_set_dispatcher, AchiDispatcher}};

use crate::{config::BLOCK_SIZE, devices::{buffer_cache::BufferCache, BlockDevice, DevId, Device, DeviceMajor, DeviceMeta, DeviceType}, drivers::block::{loongarch::ahci_blk, BLK_ID}, sync::UPSafeCell, timer::{get_current_time_duration, get_current_time_ms}};

// 等待数毫秒
fn ahci_mdelay(ms: u32) {
    let target = get_current_time_ms() + ms as usize;
    while(get_current_time_ms() < target) {
        core::hint::spin_loop();
    }
}

// 同步dcache中所有cached和uncached访存请求
fn ahci_sync_dcache() {
    unsafe {
        core::arch::asm!("dbar 0");
    }
}

// 分配按align字节对齐的内存
fn ahci_malloc_align(size: u64, align: u32) -> u64 {
    let ptr = Global.allocate(Layout::from_size_align(size as usize, align as usize).unwrap()).unwrap();
    ptr.addr().get() as u64
}
 
// 物理地址转换为uncached虚拟地址
fn ahci_phys_to_uncached(pa: u64) -> u64 {
    pa | 0x8000_0000_0000_0000
}

// cached虚拟地址转换为物理地址
// ahci dma可以接受64位的物理地址
fn ahci_virt_to_phys(va: u64) -> u64 {
    va & ((1 << Constant::PA_WIDTH) - 1)
}

pub struct AchiBlock {
    blk: UPSafeCell<ahci_device>,
    meta: DeviceMeta,
}

impl AchiBlock {
    pub fn new() -> Self {
        unsafe { ahci_set_dispatcher(
            AchiDispatcher {
                print: hal::console::_print,
                mdelay: ahci_mdelay,
                sync_dcache: ahci_sync_dcache,
                malloc_align: ahci_malloc_align,
                phys_to_uncached: ahci_phys_to_uncached,
                virt_to_phys: ahci_virt_to_phys
            }
        ); }
        let mut ahci_block: ahci_device = unsafe { core::mem::zeroed() };
        ahci_init(&mut ahci_block);
        Self { 
            blk: UPSafeCell::new(ahci_block), 
            meta: DeviceMeta { 
                dev_id: DevId {
                    major: DeviceMajor::Block,
                    minor: BLK_ID.fetch_add(1, Ordering::Relaxed),
                }, 
                name: "sda1".to_string(), 
                need_mapping: false, 
                mmio_ranges: vec![],
                irq_no: None, 
                dtype: DeviceType::Block 
            }
        }
    }
}

unsafe impl Sync for AchiBlock {}
unsafe impl Send for AchiBlock {}

impl BlockDevice for AchiBlock {
    fn size(&self) -> u64 {
        self.blk.exclusive_access().blk_dev.lba * (BLOCK_SIZE as u64)
    }

    fn buffer_cache(&self) -> Option<alloc::sync::Arc<BufferCache>> {
        // Some(self.buffer_cache.clone())
        None
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn direct_read_block(&self, block_id: usize, buf: &mut [u8]) {
        let size_round_up = ((buf.len() + self.block_size() - 1) / self.block_size()) * self.block_size();
        let blkcnt = size_round_up / self.block_size();
        let mut new_buf = vec![0u8; size_round_up];
        ahci_sata_read_common(
            self.blk.exclusive_access(),
            block_id as u64, blkcnt as u32, new_buf.as_mut_ptr()
        );
        buf.copy_from_slice(&new_buf[0..buf.len()]);
    }

    fn direct_write_block(&self, block_id: usize, buf: &[u8]) {
        let size_round_up = ((buf.len() + self.block_size() - 1) / self.block_size()) * self.block_size();
        let blkcnt = size_round_up / self.block_size();
        let last_blk_id  = block_id + blkcnt - 1;
        let mut new_buf = vec![0u8; size_round_up];
        ahci_sata_read_common(
            self.blk.exclusive_access(),
            last_blk_id as u64, 1u32, new_buf[size_round_up - self.block_size()..].as_mut_ptr()
        );
        new_buf[0..buf.len()].copy_from_slice(buf);
        ahci_sata_write_common(
            self.blk.exclusive_access(),
            block_id as u64, blkcnt as u32, new_buf.as_mut_ptr()
        );
    }
}

impl Device for AchiBlock {
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
