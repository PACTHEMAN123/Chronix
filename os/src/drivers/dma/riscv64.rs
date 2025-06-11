use core::ptr::NonNull;

use alloc::{format, vec::Vec};
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddr}, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::PageTableHal, println};
use log::info;
use virtio_drivers::BufferDirection;

use crate::{mm::{allocator::{frames_alloc, frames_alloc_clean, frames_dealloc}, vm::{KernVmSpaceHal, PageFaultAccessType, UserVmSpaceHal}, FrameTracker, KVMSPACE}, sync::UPSafeCell, task::current_task};

use super::VirtioHal;

unsafe impl virtio_drivers::Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection,) -> (virtio_drivers::PhysAddr, NonNull<u8>) {
        info!("dma_alloc");
        let frame = frames_alloc_clean(pages).unwrap();
        let ppn_base = frame.range_ppn.start;
        core::mem::forget(frame);
        let pa: PhysAddr = ppn_base.start_addr();
        (pa.0, NonNull::new(pa.get_mut::<u8>()).unwrap())
    }

    unsafe fn dma_dealloc(paddr: virtio_drivers::PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        info!("dma_dealloc");
        let pa = PhysAddr::from(paddr);
        let ppn_base: PhysPageNum = pa.floor();
        frames_dealloc(ppn_base..ppn_base+pages);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: virtio_drivers::PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(PhysAddr::from(paddr).get_mut::<u8>()).unwrap()
    }

    unsafe fn share(
        buffer: NonNull<[u8]>,
        direction: BufferDirection,
    ) -> virtio_drivers::PhysAddr {
        let buffer = buffer.as_ref();
        let pages = (buffer.len() - 1 + Constant::PAGE_SIZE) >> Constant::PAGE_SIZE_BITS;
        let frames = frames_alloc(pages).unwrap();
        match direction {
            BufferDirection::DriverToDevice |
            BufferDirection::Both => {
                frames.range_ppn.get_slice_mut()[..buffer.len()].copy_from_slice(buffer);
            }
            BufferDirection::DeviceToDriver => {}
        }
        let pa = frames.range_ppn.start.start_addr().0;
        core::mem::forget(frames);
        pa
    }

    unsafe fn unshare(
        paddr: virtio_drivers::PhysAddr,
        mut buffer: NonNull<[u8]>,
        direction: BufferDirection,
    ) {
        let buffer = buffer.as_mut();
        let ppn_start = PhysAddr::from(paddr).floor();
        let ppn_end = PhysAddr::from(paddr + buffer.len()).ceil();
        let range_ppn = ppn_start..ppn_end;
        match direction {
            BufferDirection::DeviceToDriver |
            BufferDirection::Both => {
                buffer.copy_from_slice(&range_ppn.get_slice()[..buffer.len()]);
            }
            BufferDirection::DriverToDevice => {}
        }
        frames_dealloc(range_ppn);
    }
}
