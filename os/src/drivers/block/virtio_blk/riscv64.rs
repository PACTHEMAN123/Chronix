//! VirtIO block device driver

use crate::devices::BlockDevice;
use crate::config::BLOCK_SIZE;
use crate::mm::allocator::{frames_alloc_clean, frames_dealloc, FrameAllocator};
use crate::mm::{FrameTracker, PageTable, INIT_VMSPACE};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use hal::addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, VirtAddr};
use hal::constant::{Constant, ConstantsHal};
use hal::pagetable::PageTableHal;
use hal::vm::{KernVmSpaceHal, UserVmSpaceHal};
use lazy_static::*;

use alloc::{string::ToString, sync::Arc};
use core::ptr::NonNull;

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};
use virtio_drivers::BufferDirection;

use log::*;

const VIRTIO0: usize = 0x10001000 | Constant::KERNEL_ADDR_SPACE.start;

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<VirtioHal, MmioTransport>>);

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = UPSafeCell::new(Vec::new());
}

impl BlockDevice for VirtIOBlock {

    fn size(&self) -> u64 {
        self.0
            .exclusive_access()
            .capacity() * (BLOCK_SIZE as u64)
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
    
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .exclusive_access()
            .read_blocks(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .exclusive_access()
            .write_blocks(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            let header = core::ptr::NonNull::new(VIRTIO0 as *mut VirtIOHeader).unwrap();
            let transport = MmioTransport::new(header, 4096).unwrap();
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal, MmioTransport>::new(transport).expect("failed to create blk driver"),
            ))
        }
    }
}

pub struct VirtioHal;

unsafe impl virtio_drivers::Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection,) -> (virtio_drivers::PhysAddr, NonNull<u8>) {
        info!("dma_alloc");
        let mut ppn_base = PhysPageNum(0);
        for i in 0..pages {
            let frame = frames_alloc_clean(1).unwrap();
            if i == 0 {
                ppn_base = frame.range_ppn.start;
            }
            assert_eq!(frame.range_ppn.start.0, ppn_base.0 + i);
            QUEUE_FRAMES.exclusive_access().push(frame);
        }
        let pa: PhysAddr = ppn_base.start_addr();
        (pa.0, NonNull::new(pa.get_mut::<u8>()).unwrap())
    }

    unsafe fn dma_dealloc(paddr: virtio_drivers::PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        info!("dma_dealloc");
        let pa = PhysAddr::from(paddr);
        let mut ppn_base: PhysPageNum = pa.floor();
        for _ in 0..pages {
            frames_dealloc(ppn_base..ppn_base+1);
            ppn_base += 1;
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: virtio_drivers::PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(PhysAddr::from(paddr).get_mut::<u8>()).unwrap()
    }

    unsafe fn share(
        buffer: NonNull<[u8]>,
        _direction: BufferDirection,
    ) -> virtio_drivers::PhysAddr {
        // use kernel space pagetable to get the physical address
        let pa = INIT_VMSPACE.lock().translate_va(VirtAddr::from(buffer.as_ptr() as *const u8 as usize)).unwrap();
        
        pa.0
    }

    unsafe fn unshare(
        _paddr: virtio_drivers::PhysAddr,
        _buffer: NonNull<[u8]>,
        _direction: BufferDirection,
    ) {
    }
}
