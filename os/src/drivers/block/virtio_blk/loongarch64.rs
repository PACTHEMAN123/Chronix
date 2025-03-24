//! VirtIO block device driver

use crate::devices::BlockDevice;
use crate::config::BLOCK_SIZE;
use crate::mm::allocator::{frames_alloc, frames_alloc_clean, frames_dealloc, FrameAllocator};
use crate::mm::{FrameTracker, PageTable, INIT_VMSPACE};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use hal::addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, VirtAddr};
use hal::constant::{Constant, ConstantsHal};
use hal::pagetable::PageTableHal;
use hal::println;
use crate::mm::vm::{KernVmSpaceHal, UserVmSpaceHal};
use lazy_static::*;

use alloc::{string::ToString, sync::Arc};
use virtio_drivers::transport::pci::bus::{BarInfo, Cam, Command, DeviceFunction, MemoryBarType, MmioCam, PciRoot};
use core::ptr::NonNull;

use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::pci::PciTransport;
use virtio_drivers::transport::{DeviceType, Transport};
use virtio_drivers::BufferDirection;

use log::*;

const VIRTIO0: usize = 0x8000_0000_2000_0000;

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<VirtioHal, PciTransport>>);

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
        let mut allocator = PciMemory32Allocator::for_pci_ranges(0x4000_0000, 0x8000_0000);
        let mut root = PciRoot::new(unsafe { 
            MmioCam::new(VIRTIO0 as *mut u8, Cam::Ecam)
        });
        let mut device_function = None;
        for (df, dfi) in root.enumerate_bus(0) {
            if dfi.class == 1 {
                device_function = Some(df);
                break;
            }
        }
        let device_function = device_function.expect("block device not found");
        for (i, info) in root.bars(device_function).unwrap().into_iter().enumerate() {
            let Some(info) = info else { continue };
            info!("BAR {}: {}", i, info);
            if let BarInfo::Memory {
                address_type, size, ..
            } = info {
                match address_type {
                    MemoryBarType::Width32 => {
                        if size > 0 {
                            let addr = allocator.allocate_memory_32(size);
                            info!("Allocated address: {:#x}", addr);
                            root.set_bar_32(device_function, i as u8, addr as u32);
                        }
                    },
                    MemoryBarType::Width64 => {
                        if size > 0 {
                            let addr = allocator.allocate_memory_32(size);
                            info!("Allocated address: {:#x}", addr);
                            root.set_bar_64(device_function, i as u8, addr as u64);
                        }
                    },
                    _ => panic!("Memory BAR address type {:?} not supported.", address_type),
                }
            }
            
        }
        root.set_command(
            device_function,
            Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
        );
        let (status, command) = root.get_status_command(device_function);
        info!(
            "Allocated BARs and enabled device, status {:?} command {:?}",
            status, command
        );
        // dump_bar_contents(&mut root, device_function, 4);
        let mut transport = PciTransport::new::<VirtioHal, _>(&mut root, device_function).unwrap();
        info!(
            "Detected virtio PCI device with device type {:?}, features {:#018x}",
            transport.device_type(),
            transport.read_device_features(),
        );
        Self(UPSafeCell::new(
            VirtIOBlk::<VirtioHal, PciTransport>::new(transport).expect("failed to create blk driver"),
        ))
    }
}

#[allow(unused)]
fn dump_bar_contents(
    root: &mut PciRoot<MmioCam>,
    device_function: DeviceFunction,
    bar_index: u8,
) {
    let bar_info = root.bar_info(device_function, bar_index).unwrap();
    println!("Dumping bar {}: {}", bar_index, bar_info);
    if let BarInfo::Memory { address, size, .. } = bar_info {
        let start = (address | 0x8000_0000_0000_0000) as *const u8;
        unsafe {
            let mut buf = [0u8; 32];
            for i in 0..size / 32 {
                let ptr = start.add(i as usize * 32);
                core::ptr::copy(ptr, buf.as_mut_ptr(), 32);
                if buf.iter().any(|b| *b != 0xff) {
                    println!("  {:?}: {:x?}", ptr, buf);
                }
            }
        }
    }
    println!("End of dump");
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
struct PciMemory32Allocator {
    start: u32,
    end: u32,
}

impl PciMemory32Allocator {
    /// Creates a new allocator based on the ranges property of the given PCI node.
    pub fn for_pci_ranges(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
        }
    }

    /// Allocates a 32-bit memory address region for a PCI BAR of the given power-of-2 size.
    ///
    /// It will have alignment matching the size. The size must be a power of 2.
    pub fn allocate_memory_32(&mut self, size: u32) -> u32 {
        assert!(size.is_power_of_two());
        let allocated_address = align_up(self.start, size);
        assert!(allocated_address + size <= self.end);
        self.start = allocated_address + size;
        allocated_address
    }
}

const fn align_up(value: u32, alignment: u32) -> u32 {
    ((value - 1) | (alignment - 1)) + 1
}
