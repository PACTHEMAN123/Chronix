use lazy_static::*;
use core::ptr::NonNull;
use alloc::vec::Vec;
use hal::addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, VirtAddr};
use log::info;
use virtio_drivers::BufferDirection;
use crate::{mm::{allocator::{frames_alloc_clean, frames_dealloc}, vm::KernVmSpaceHal, FrameTracker, KVMSPACE}, sync::UPSafeCell};

use super::VirtioHal;

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = UPSafeCell::new(Vec::new());
}

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
        NonNull::new((paddr | 0x8000_0000_0000_0000) as *mut u8).unwrap()
    }

    unsafe fn share(
        buffer: NonNull<[u8]>,
        _direction: BufferDirection,
    ) -> virtio_drivers::PhysAddr {
        buffer.as_ptr() as *const u8 as usize & 0xffff_ffff
    }

    unsafe fn unshare(
        _paddr: virtio_drivers::PhysAddr,
        _buffer: NonNull<[u8]>,
        _direction: BufferDirection,
    ) {
    }
}