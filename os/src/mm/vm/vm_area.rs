use core::ops::{DerefMut, Range};

use alloc::{alloc::Global, collections::btree_map::{BTreeMap, Keys}, sync::Arc};
use log::info;

use crate::{arch::riscv64::sfence_vma_vaddr, config::{KERNEL_STACK_BOTTOM, KERNEL_STACK_TOP}}; 
use crate::config::{KERNEL_ADDR_OFFSET, KERNEL_STACK_SIZE, PAGE_SIZE};
use crate::mm::{PageTableEntry, allocator::{frame_alloc, frame_alloc_clean, FrameTracker}, page_table::{PTEFlags, PageTable}, smart_pointer::StrongArc, address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}};
use bitflags::bitflags;

use super::PageFaultAccessType;

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
            unsafe { sfence_vma_vaddr(vpn.into()) };
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

    fn map_range_to(&self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>, start_ppn: PhysPageNum) {
        range_vpn
        .enumerate()
        .for_each(|(i, vpn)| {
            let ppn = PhysPageNum(start_ppn.0 + i);
            page_table.map(vpn, ppn, (*self.perm()).into());
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

    fn add_allocated_frame(&mut self, vpn: VirtPageNum, frame: FrameTracker);

    fn remove_allocated_frame(&mut self, vpn: VirtPageNum);

    fn map_range_and_alloc_frames(&mut self, page_table: &mut PageTable, range: Range<VirtPageNum>) {
        range
        .for_each(|vpn| {
            let frame = frame_alloc_clean().unwrap();
            page_table.map(vpn, frame.ppn, (*self.perm()).into());
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
            let pte = page_table.find_leaf_pte(vpn).unwrap();
            log::trace!(
                "[origin pte:{:?}, new_flag:{:?}]",
                pte.flags(),
                pte.flags().union(pte_flags)
            );
            pte.set_flags(pte.flags().union(pte_flags));
            unsafe { sfence_vma_vaddr(vpn.into()) };
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
    pub pages: BTreeMap<VirtPageNum, StrongArc<FrameTracker>, Global>,
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
                unsafe { sfence_vma_vaddr(vpn.into()); }
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
                self.map_range_to(page_table, vpn..vpn+1, frame.ppn);
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
                range
                    .for_each(|vpn| {
                        // try_unmap, because of lazy allocation
                        let _ = page_table.try_unmap(vpn);
                        self.remove_allocated_frame(vpn);
                    });
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
    
    fn add_allocated_frame(&mut self, vpn: VirtPageNum, frame: FrameTracker) {
        self.pages.insert(vpn, StrongArc::new(frame));
    }
    
    fn remove_allocated_frame(&mut self, vpn: VirtPageNum) {
        self.pages.remove(&vpn);
    }
}

pub struct UserVmAreaFrameIter<'a> {
    inner: Keys<'a, VirtPageNum, StrongArc<FrameTracker>>
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
        match page_table.find_pte(vpn) {
            Some(pte) if pte.is_valid() => {
                // Cow
                let frame = self.pages.get(&vpn)?;
                if frame.get_rc() == 1 {
                    self.perm_mut().remove(MapPerm::C);
                    self.perm_mut().insert(MapPerm::W);
                    pte.set_flags(PTEFlags::from(self.map_perm) | PTEFlags::V);
                    unsafe { sfence_vma_vaddr(vpn.into()) };
                    Some(())
                } else {
                    let new_frame = StrongArc::new(frame_alloc_clean()?);
                    let new_ppn = new_frame.ppn;

                    let old_data = &frame.ppn.to_kern().get_bytes_array()[..];
                    new_ppn.to_kern().get_bytes_array().copy_from_slice(old_data);
                    
                    *self.pages.get_mut(&vpn)? = new_frame;

                    self.perm_mut().remove(MapPerm::C);
                    self.perm_mut().insert(MapPerm::W);
                    *pte = PageTableEntry::new(new_ppn, PTEFlags::from(self.map_perm) | PTEFlags::V);
                    
                    unsafe { sfence_vma_vaddr(vpn.into()) };
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
                        unsafe { crate::arch::riscv64::sfence_vma_vaddr(vpn.into()) };
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
    pub pages: BTreeMap<VirtPageNum, FrameTracker>,
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

    fn add_allocated_frame(&mut self, vpn: VirtPageNum, frame: FrameTracker) {
        self.pages.insert(vpn, frame);
    }

    fn remove_allocated_frame(&mut self, vpn: VirtPageNum) {
        self.pages.remove(&vpn);
    }
}

pub struct KernelVmAreaFrameIter<'a> {
    inner: Keys<'a, VirtPageNum, FrameTracker>
}

impl<'a> Iterator for KernelVmAreaFrameIter<'a> {
    type Item = &'a VirtPageNum;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}