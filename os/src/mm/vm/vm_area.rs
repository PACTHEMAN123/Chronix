use core::ops::{DerefMut, Range};

use alloc::{alloc::Global, collections::btree_map::{BTreeMap, Keys}, sync::Arc};
use hal::instruction::InstructionHal;
use log::info;

use crate::{arch::Instruction, config::{KERNEL_STACK_BOTTOM, KERNEL_STACK_TOP}, mm::{allocator::{frames_alloc_clean, FrameRangeTracker}, RangeKpnData, ToRangeKpn}}; 
use crate::config::{KERNEL_ADDR_OFFSET, KERNEL_STACK_SIZE, PAGE_SIZE};
use crate::mm::{PageTableEntry, allocator::{frame_alloc, frame_alloc_clean, FrameTracker}, page_table::{PTEFlags, PageTable}, smart_pointer::StrongArc, address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}};
use bitflags::bitflags;

use super::{PageFaultAccessType, VpnPageRangeIter, VpnPageRangeWithAllocIter};

bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    pub struct MapPerm: u16 {
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// Executable
        const X = 1 << 3;
        /// User-mode accessible
        const U = 1 << 4;
        /// Copy On Write
        const C = 1 << 8;

        /// Read-write
        const RW = Self::R.bits() | Self::W.bits();
        /// Read-execute
        const RX = Self::R.bits() | Self::X.bits();
        /// Reserved
        const WX = Self::W.bits() | Self::X.bits();
        /// Read-write-execute
        const RWX = Self::R.bits() | Self::W.bits() | Self::X.bits();

        /// User Write-only
        const UW = Self::U.bits() | Self::W.bits();
        /// User Read-write
        const URW = Self::U.bits() | Self::RW.bits();
        /// Uer Read-execute
        const URX = Self::U.bits() | Self::RX.bits();
        /// Reserved
        const UWX = Self::U.bits() | Self::WX.bits();
        /// User Read-write-execute
        const URWX = Self::U.bits() | Self::RWX.bits();
        
        /// Read freely, copy on write
        const RC = Self::R.bits() | Self::C.bits();
        /// User Read-COW
        const URC = Self::U.bits() | Self::RC.bits();
    }
}

impl From<MapPerm> for PTEFlags {
    fn from(value: MapPerm) -> Self {
        Self::from_bits_truncate(value.bits)
    }
}

#[allow(missing_docs)]
pub trait VmArea: Sized
{
    fn split_off(&mut self, p: VirtPageNum) -> Self;

    fn range_va(&self) -> &Range<VirtAddr>;

    fn range_va_mut(&mut self) -> &mut Range<VirtAddr>;

    fn start_va(&self) -> VirtAddr {
        self.range_va().start
    }

    fn end_va(&self) -> VirtAddr {
        self.range_va().end
    }

    fn start_vpn(&self) -> VirtPageNum {
        self.start_va().floor()
    }

    fn end_vpn(&self) -> VirtPageNum {
        self.end_va().ceil()
    }

    fn range_vpn(&self) -> Range<VirtPageNum> {
        self.start_vpn()..self.end_vpn()
    }

    fn set_range_va(&mut self, range_va: Range<VirtAddr>) {
        *self.range_va_mut() = range_va
    }

    fn flush(&mut self) {
        let range_vpn = self.range_vpn();
        for vpn in range_vpn {
            unsafe { Instruction::tlb_flush_addr(vpn.into()) };
        }
    }

    fn perm(&self) -> &MapPerm;

    fn perm_mut(&mut self) -> &mut MapPerm;

    fn set_perm(&mut self, perm: MapPerm) {
        *self.perm_mut() = perm;
    }

    fn set_perm_flush(&mut self, perm: MapPerm) {
        *self.perm_mut() = perm;
        self.flush();
    }

    fn map_range_to(&self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>, mut start_ppn: PhysPageNum) {
        VpnPageRangeIter::new(range_vpn)
        .for_each(|(vpn, level)| {
            let ppn = PhysPageNum(start_ppn.0);
            start_ppn += level.page_count();
            page_table.map(vpn, ppn, (*self.perm()).into(), level);
        });
    }

    fn map_range(&mut self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>);

    fn unmap_range(&mut self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>);

    fn map(&mut self, page_table: &mut PageTable) {
        self.map_range(page_table, self.range_vpn());
    }

    fn unmap(&mut self, page_table: &mut PageTable) {
        self.unmap_range(page_table, self.range_vpn());
    }

    fn shrink_to(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        self.unmap_range(page_table, new_end..self.end_vpn());
        *self.range_va_mut() = self.start_vpn().into()..new_end.into();
    }

    fn append_to(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        self.map_range(page_table, self.end_vpn()..new_end);
        *self.range_va_mut() = self.start_vpn().into()..new_end.into();
    }

    fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        let mut start: usize = 0;
        let len = data.len();
        for vpn in self.range_vpn() {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(vpn)
                .unwrap()
                .ppn()
                .to_kern()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
        }
    }

}

#[allow(missing_docs)]
pub trait VmAreaFrameExt: VmArea {
    type FrameIter<'a>: Iterator<Item = &'a VirtPageNum> where Self: 'a;

    fn allocated_frames_iter<'a>(&'a self) -> Self::FrameIter<'a>;

    fn add_allocated_frame(&mut self, vpn: VirtPageNum, frame: FrameRangeTracker);

    fn remove_allocated_frame(&mut self, vpn: VirtPageNum);

    fn map_range_and_alloc_frames(&mut self, page_table: &mut PageTable, range: Range<VirtPageNum>) {
        VpnPageRangeWithAllocIter::new(range)
        .for_each(|(vpn, frame, level)| {
            frame.clean();
            page_table.map(vpn, frame.range_ppn.start, (*self.perm()).into(), level);
            self.add_allocated_frame(vpn, frame);
        });
    }

    fn unmap_range_and_dealloc_frames(&mut self, page_table: &mut PageTable, range: Range<VirtPageNum>) {
        range
        .for_each(|vpn| {
            page_table.unmap(vpn);
            self.remove_allocated_frame(vpn);
        });
    }

    fn map_and_alloc_frames(&mut self, page_table: &mut PageTable) {
        self.map_range_and_alloc_frames(page_table, self.range_vpn());
    }

    fn unmap_and_dealloc_frames(&mut self, page_table: &mut PageTable) {
        self.unmap_range_and_dealloc_frames(page_table, self.range_vpn());
    }

    fn set_perm_and_flush_allocated_frames(&mut self, page_table: &mut PageTable, perm: MapPerm) {
        self.set_perm(perm);
        let pte_flags = perm.into();
        // NOTE: should flush pages that already been allocated, page fault handler will
        // handle the permission of those unallocated pages
        for &vpn in self.allocated_frames_iter() {
            let (pte, _) = page_table.find_leaf_pte(vpn).unwrap();
            log::trace!(
                "[origin pte:{:?}, new_flag:{:?}]",
                pte.flags(),
                pte.flags().union(pte_flags)
            );
            pte.set_flags(pte.flags().union(pte_flags));
            unsafe { Instruction::tlb_flush_addr(vpn.into()) };
        }
    }
}

#[allow(missing_docs)]
pub trait VmAreaPageFaultExt: VmArea {
    fn handle_page_fault(&mut self, 
        page_table: &mut PageTable, 
        vpn: VirtPageNum,
        access_type: PageFaultAccessType
    ) -> Option<()>;
}

#[allow(missing_docs)]
pub trait VmAreaCowExt: VmArea {
    fn clone_cow(&mut self, page_table: &mut PageTable) -> Result<Self, Self>;
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserVmAreaType {
    Elf, Stack, Heap, TrapContext
}

/// User's Virtual Memory Area
#[allow(missing_docs)]
pub struct UserVmArea {
    range_va: Range<VirtAddr>,
    pub pages: BTreeMap<VirtPageNum, StrongArc<FrameRangeTracker>, Global>,
    pub map_perm: MapPerm,
    pub vma_type: UserVmAreaType,
}

#[allow(missing_docs)]
impl UserVmArea {
    pub fn new(range_va: Range<VirtAddr>, map_perm: MapPerm, vma_type: UserVmAreaType) -> Self {
        let range_va = range_va.start.floor().into()..range_va.end.ceil().into();
        Self {
            range_va,
            pages: BTreeMap::new_in(Global),
            map_perm,
            vma_type
        }
    }

}

impl VmAreaCowExt for UserVmArea {
    fn clone_cow(&mut self, page_table: &mut PageTable) -> Result<Self, Self> {
        // note: trap context cannot supprt COW
        if self.vma_type == UserVmAreaType::TrapContext {
            return Err(self.clone());
        }
        if self.perm().contains(MapPerm::W) {
            self.perm_mut().insert(MapPerm::C);
            self.perm_mut().remove(MapPerm::W);
            for &vpn in self.allocated_frames_iter() {
                page_table.update_perm(vpn, (*self.perm()).into());
                unsafe { Instruction::tlb_flush_addr(vpn.into()); }
            }
        } else {
            self.perm_mut().insert(MapPerm::C);
        }
        Ok(Self {
            range_va: self.range_va.clone(), 
            pages: self.pages.clone(), 
            map_perm: self.map_perm.clone(), 
            vma_type: self.vma_type.clone() 
        })
    }
}

impl Clone for UserVmArea {
    fn clone(&self) -> Self {
        Self { 
            range_va: self.range_va.clone(), 
            pages: BTreeMap::new_in(Global), 
            map_perm: self.map_perm.clone(), 
            vma_type: self.vma_type.clone() 
        }
    }
}

impl VmArea for UserVmArea {
    fn range_va(&self) -> &Range<VirtAddr> {
        &self.range_va
    }

    fn range_va_mut(&mut self) -> &mut Range<VirtAddr> {
        &mut self.range_va
    }

    fn perm(&self) -> &MapPerm {
        &self.map_perm
    }

    fn perm_mut(&mut self) -> &mut MapPerm {
        &mut self.map_perm
    }
    
    fn map_range(&mut self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>) {
        if self.perm().contains(MapPerm::C) {
            for (&vpn, frame) in self.pages.iter() {
                self.map_range_to(page_table, vpn..vpn+1, frame.range_ppn.start);
            }
        } else {
            match self.vma_type {
                UserVmAreaType::Elf
                | UserVmAreaType::TrapContext => self.map_range_and_alloc_frames(page_table, range_vpn),
                UserVmAreaType::Stack 
                | UserVmAreaType::Heap => {}
            }
        }
    }
    
    fn unmap_range(&mut self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>) {
        self.unmap_range_and_dealloc_frames(page_table, range_vpn);
    }

    fn split_off(&mut self, p: VirtPageNum) -> Self {
        debug_assert!(self.range_va.contains(&p.into()));
        let ret = Self {
            range_va: p.into()..self.end_va(),
            pages: self.pages.split_off(&p),
            map_perm: self.map_perm,
            vma_type: self.vma_type
        };
        self.range_va = self.start_va()..p.into();
        ret
    }
}

impl VmAreaFrameExt for UserVmArea {
    type FrameIter<'a> = UserVmAreaFrameIter<'a>;
    
    fn allocated_frames_iter<'a>(&'a self) -> Self::FrameIter<'a> {
        UserVmAreaFrameIter{
            inner: self.pages.keys()
        }
    }

    fn unmap_range_and_dealloc_frames(&mut self, page_table: &mut PageTable, range: Range<VirtPageNum>) {
        match self.vma_type {
            UserVmAreaType::Heap
            | UserVmAreaType::Stack => {
                let mut mid = self.pages.split_off(&range.start);
                let mut right = mid.split_off(&range.end);
                for &vpn in mid.keys() {
                    page_table.unmap(vpn);
                }
                self.pages.append(&mut right);
            },
            _ => {
                range
                    .for_each(|vpn| {
                        page_table.unmap(vpn);
                        self.remove_allocated_frame(vpn);
                    });
            }
        }
    }
    
    fn add_allocated_frame(&mut self, vpn: VirtPageNum, frame: FrameRangeTracker) {
        self.pages.insert(vpn, StrongArc::new(frame));
    }
    
    fn remove_allocated_frame(&mut self, vpn: VirtPageNum) {
        self.pages.remove(&vpn);
    }
}

pub struct UserVmAreaFrameIter<'a> {
    inner: Keys<'a, VirtPageNum, StrongArc<FrameRangeTracker>>
}

impl<'a> Iterator for UserVmAreaFrameIter<'a> {
    type Item = &'a VirtPageNum;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl VmAreaPageFaultExt for UserVmArea {
    fn handle_page_fault(&mut self, 
        page_table: &mut PageTable, 
        vpn: VirtPageNum,
        access_type: PageFaultAccessType
    ) -> Option<()> {
        if !access_type.can_access(*self.perm()) {
            log::warn!(
                "[VmArea::handle_page_fault] permission not allowed, perm:{:?}",
                self.perm()
            );
            return None;
        }
        match page_table.find_leaf_pte(vpn) {
            Some((pte, mut level)) if pte.is_valid() => {
                // Cow
                let frame = self.pages.get(&vpn)?;
                if frame.get_rc() == 1 {
                    self.perm_mut().remove(MapPerm::C);
                    self.perm_mut().insert(MapPerm::W);
                    pte.set_flags(PTEFlags::from(self.map_perm) | PTEFlags::V);
                    unsafe { Instruction::tlb_flush_addr(vpn.into()) };
                    Some(())
                } else {
                    let new_frame = StrongArc::new(
                        loop {
                            if let Some(frame) = frames_alloc_clean(level.page_count()) {
                                break Some(frame);
                            } else if level.lowest() {
                                break None;
                            }
                            level = level.lower();
                        }?
                    );
                    let new_range_ppn = new_frame.range_ppn.clone();

                    let old_data = &frame.range_ppn.to_kern().get_slice::<u8>();
                    new_range_ppn.to_kern().get_slice::<u8>().copy_from_slice(old_data);
                    
                    *self.pages.get_mut(&vpn)? = new_frame;

                    self.perm_mut().remove(MapPerm::C);
                    self.perm_mut().insert(MapPerm::W);
                    *pte = PageTableEntry::new(new_range_ppn.start, PTEFlags::from(self.map_perm) | PTEFlags::V);
                    
                    unsafe { Instruction::tlb_flush_addr(vpn.into()) };
                    Some(())
                }
            }
            _ => {
                match self.vma_type {
                    UserVmAreaType::Elf
                    | UserVmAreaType::TrapContext => {
                        return None
                    },
                    UserVmAreaType::Stack
                    | UserVmAreaType::Heap => {
                        self.map_range_and_alloc_frames(page_table, vpn..vpn+1);
                        unsafe { Instruction::tlb_flush_addr(vpn.into()) };
                        return Some(());
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum KernelVmAreaType {
    Text, Rodata, Data, Bss, PhysMem, MemMappedReg, KernelStack
}

/// Kernel's Virtual Memory Area
#[allow(missing_docs)]
pub struct KernelVmArea {
    range_va: Range<VirtAddr>,
    pub pages: BTreeMap<VirtPageNum, FrameRangeTracker>,
    pub map_perm: MapPerm,
    pub vma_type: KernelVmAreaType,
}

#[allow(missing_docs)]
impl KernelVmArea {

    pub fn new(range_va: Range<VirtAddr>, map_perm: MapPerm, vma_type: KernelVmAreaType) -> Self {
        let range_va = (VirtAddr(range_va.start.0)).floor().into() ..
                                        (VirtAddr(range_va.end.0)).ceil().into();
        Self {
            range_va,
            pages: BTreeMap::new(),
            map_perm,
            vma_type
        }
    }

    pub fn map_range_highly(&mut self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>) {
        self.map_range_to(page_table, range_vpn, PhysPageNum(self.start_vpn().0 & !(KERNEL_ADDR_OFFSET >> 12)));
    }
}

impl VmArea for KernelVmArea {
    fn range_va(&self) -> &Range<VirtAddr> {
        &self.range_va
    }

    fn range_va_mut(&mut self) -> &mut Range<VirtAddr> {
        &mut self.range_va
    }

    fn perm(&self) -> &MapPerm {
        &self.map_perm
    }

    fn perm_mut(&mut self) -> &mut MapPerm {
        &mut self.map_perm
    }
    
    fn map_range(&mut self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>) {
        match self.vma_type {
            KernelVmAreaType::Bss |
            KernelVmAreaType::Data |
            KernelVmAreaType::MemMappedReg |
            KernelVmAreaType::PhysMem |
            KernelVmAreaType::Rodata |
            KernelVmAreaType::Text => self.map_range_highly(page_table, range_vpn),
            KernelVmAreaType::KernelStack => {
                self.map_range_to(
                    page_table, 
                    KERNEL_STACK_BOTTOM.into()..KERNEL_STACK_TOP.into(),
                    PhysPageNum(range_vpn.start.0 & (KERNEL_ADDR_OFFSET >> 12))
                );
            },
        }
    }
    
    fn unmap_range(&mut self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>) {

        match self.vma_type {
            KernelVmAreaType::Bss |
            KernelVmAreaType::Data |
            KernelVmAreaType::MemMappedReg |
            KernelVmAreaType::PhysMem |
            KernelVmAreaType::Rodata |
            KernelVmAreaType::Text => {
                range_vpn
                .for_each(|vpn| {
                    page_table.unmap(vpn);
                });
            },
            KernelVmAreaType::KernelStack => self.unmap_range_and_dealloc_frames(page_table, range_vpn),
        }
        
    }

    fn split_off(&mut self, p: VirtPageNum) -> Self {
        debug_assert!(self.range_va.contains(&p.into()));
        let ret = Self {
            range_va: p.into()..self.end_va(),
            pages: self.pages.split_off(&p),
            map_perm: self.map_perm,
            vma_type: self.vma_type
        };
        self.range_va = self.start_va()..p.into();
        ret
    }
}

impl VmAreaFrameExt for KernelVmArea {
    type FrameIter<'a> = KernelVmAreaFrameIter<'a>;

    fn allocated_frames_iter<'a>(&'a self) -> Self::FrameIter<'a> {
        KernelVmAreaFrameIter { inner: self.pages.keys() }
    }

    fn add_allocated_frame(&mut self, vpn: VirtPageNum, frame: FrameRangeTracker) {
        self.pages.insert(vpn, frame);
    }

    fn remove_allocated_frame(&mut self, vpn: VirtPageNum) {
        self.pages.remove(&vpn);
    }
}

pub struct KernelVmAreaFrameIter<'a> {
    inner: Keys<'a, VirtPageNum, FrameRangeTracker>
}

impl<'a> Iterator for KernelVmAreaFrameIter<'a> {
    type Item = &'a VirtPageNum;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}