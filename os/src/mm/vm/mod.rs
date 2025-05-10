use core::ops::Range;
use alloc::{alloc::Global, collections::btree_map::BTreeMap, sync::Arc, vec::Vec};

use bitflags::bitflags;
use hal::{addr::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}, instruction::{Instruction, InstructionHal}, pagetable::{MapPerm, PageTableHal}, util::smart_point::StrongArc};
use xmas_elf::{reader::Reader, ElfFile};

use crate::{fs::{shmfs::file::ShmFile, vfs::File}, sync::mutex::{spin_mutex::SpinMutex, MutexSupport}, syscall::{mm::MmapFlags, SysError, SysResult}, task::utils::AuxHeader};

use super::{allocator::{FrameAllocator, SlabAllocator}, FrameTracker, PageTable};

/// Type of Kernel's Virtual Memory Area
#[derive(Debug, Clone, Copy,  PartialEq, Eq)]
pub enum KernVmAreaType {
    ///
    Data,
    /// physical memory 
    PhysMem, 
    /// mmio
    MemMappedReg,
    /// 
    KernelStack,
    ///
    SigretTrampoline,
    ///
    VirtMemory,
    ///
    Mmap,
}

/// Type of User's Virtual Memory Area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserVmAreaType {
    /// data
    Data, 
    /// heap
    Heap, 
    /// stack
    Stack,
    /// file mmap
    Mmap,
}

#[allow(missing_docs)]
#[derive(Clone)]
pub enum UserVmFile {
    None,
    File(Arc<dyn File>),
    Shm(Arc<ShmFile>)
}

#[allow(missing_docs)]
impl UserVmFile {
    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            _ => true
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false
        }
    }

    pub fn unwrap_file(self) -> Arc<dyn File> {
        match self {
            Self::File(f) => f,
            _ => panic!("UserVmFile is not File")
        }
    }

    pub fn unwrap_shm(self) -> Arc<ShmFile> {
        match self {
            Self::Shm(shm) => shm,
            _ => panic!("UserVmFile is not Shm")
        }
    }

}

impl From<Option<Arc<dyn File>>> for UserVmFile {
    fn from(value: Option<Arc<dyn File>>) -> Self {
        match value {
            None => Self::None,
            Some(file) => Self::File(file)
        }
    }
}

#[allow(missing_docs, unused)]
pub struct UserVmArea {
    pub range_va: Range<VirtAddr>,
    pub vma_type: UserVmAreaType,
    pub map_perm: MapPerm,
    frames: BTreeMap<VirtPageNum, StrongArc<FrameTracker, SlabAllocator>>,
    /// for mmap usage
    pub file: UserVmFile,
    pub mmap_flags: MmapFlags,
    /// offset in file
    pub offset: usize,
    /// length of file
    pub len: usize,
}

#[allow(missing_docs, unused)]
impl UserVmArea {
    fn new(
        range_va: Range<VirtAddr>, 
        vma_type: UserVmAreaType, 
        map_perm: MapPerm,
    ) -> Self {
        Self {
            range_va,
            vma_type,
            map_perm,
            frames: BTreeMap::new(),
            file: UserVmFile::None,
            mmap_flags: MmapFlags::default(),
            offset: 0,
            len: 0
        }
    }

    fn new_mmap(
        range_va: Range<VirtAddr>,
        map_perm: MapPerm,
        flags: MmapFlags,
        file: UserVmFile,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            range_va,
            vma_type: UserVmAreaType::Mmap,
            map_perm,
            frames: BTreeMap::new(),
            file,
            mmap_flags: flags,
            offset,
            len
        }
    }
}

#[allow(missing_docs, unused)]
pub struct KernVmArea {
    range_va: Range<VirtAddr>,
    pub vma_type: KernVmAreaType,
    pub map_perm: MapPerm,
    pub frames: BTreeMap<VirtPageNum, StrongArc<FrameTracker, SlabAllocator>>,
    /// for mmap usage
    pub file: Option<Arc<dyn File>>,
}

#[allow(missing_docs, unused)]
impl KernVmArea {
    pub fn new(
        range_va: Range<VirtAddr>, 
        vma_type: KernVmAreaType, 
        map_perm: MapPerm
    ) -> Self {
        Self {
            range_va,
            vma_type,
            map_perm,
            frames: BTreeMap::new(),
            file: None,
        }
    }
}

bitflags! {
    /// PageFaultAccessType
    pub struct PageFaultAccessType: u8 {
        /// Read
        const READ = 1 << 0;
        /// Write
        const WRITE = 1 << 1;
        /// Execute
        const EXECUTE = 1 << 2;
    }
}

#[allow(missing_docs, unused)]
impl PageFaultAccessType {
    pub fn can_access(self, flag: MapPerm) -> bool {
        if self.contains(Self::WRITE) && !flag.contains(MapPerm::W) && !flag.contains(MapPerm::C) {
            return false;
        }
        if self.contains(Self::EXECUTE) && !flag.contains(MapPerm::X) {
            return false;
        }
        true
    }
}

#[allow(missing_docs)]
pub type StackTop = usize;
#[allow(missing_docs)]
pub type EntryPoint = usize;
#[allow(missing_docs)]
pub type MaxEndVpn = VirtPageNum;
#[allow(missing_docs)]
pub type StartPoint = VirtAddr;

#[allow(missing_docs, unused)]
pub trait KernVmSpaceHal {
    
    fn enable(&self);

    fn get_page_table(&self) -> &PageTable;

    fn new() -> Self;

    fn to_user<T: UserVmSpaceHal>(&self) -> T;

    fn push_area(&mut self, area: KernVmArea, data: Option<&[u8]>);

    fn mmap(&mut self, file: Arc<dyn File>) -> Result<VirtAddr, ()>;

    fn unmap(&mut self, va: VirtAddr) -> Result<(), ()>;

    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum>;

    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr>;

    fn handle_page_fault(&mut self, va: VirtAddr, access_type: PageFaultAccessType) -> Result<(), ()>;
}

#[allow(missing_docs, unused)]
pub trait UserVmSpaceHal: Sized {

    fn new() -> Self;

    fn get_page_table(&self) -> &PageTable;

    fn enable(&self) {
        unsafe {
            self.get_page_table().enable_low();
            Instruction::tlb_flush_all();
        }
    }

    fn map_elf<T: Reader + ?Sized>(&mut self, elf: &ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>, offset: VirtAddr) -> 
        (MaxEndVpn, StartPoint);

    fn from_elf<T: Reader + ?Sized>(elf: &ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>) -> 
        Result<(Self, StackTop, EntryPoint, Vec<AuxHeader>), SysError>;

    fn from_existed(uvm_space: &mut Self) -> Self;

    /// warning: data must must be page-aligned
    fn push_area(&mut self, area: UserVmArea, data: Option<&[u8]>) -> &mut UserVmArea;

    fn reset_heap_break(&mut self, new_brk: VirtAddr) -> VirtAddr;

    fn handle_page_fault(&mut self, va: VirtAddr, access_type: PageFaultAccessType) -> Result<(), ()>;

    fn check_free(&self, va: VirtAddr, len: usize) -> Result<(), ()>;

    fn get_area_view(&self, va: VirtAddr) -> Option<UserVmArea>;

    fn get_area_mut(&mut self, va: VirtAddr) -> Option<&mut UserVmArea>;

    fn alloc_mmap_area(&mut self, va: VirtAddr, len: usize, perm: MapPerm, flags: MmapFlags, file: Arc<dyn File>, offset: usize) -> Result<VirtAddr, SysError>;

    fn alloc_anon_area(&mut self, va: VirtAddr, len: usize, perm: MapPerm, flags: MmapFlags, id: Option<usize>) -> Result<VirtAddr, SysError>;

    fn unmap(&mut self, va: VirtAddr, len: usize) -> Result<UserVmArea, SysError>;

    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        self.get_page_table().translate_vpn(vpn)
    }

    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.get_page_table().translate_va(va)
    }
}

mod uvm;
pub use uvm::*;

mod kvm;
pub use kvm::*;
