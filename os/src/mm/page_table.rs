//! Implementation of [`PageTableEntry`] and [`PageTable`].

use core::cmp::min;
use core::ptr::slice_from_raw_parts_mut;

use crate::arch::Instruction;
use crate::config::PAGE_SIZE;

use super::allocator::{frame_alloc_clean, frame_alloc, FrameTracker};
use super::address::{KernAddr, PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::vm::KERNEL_SPACE;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use hal::instruction::InstructionHal;
use log::info;
bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u16 {
        /// Valid
        const V = 1 << 0;
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// Executable
        const X = 1 << 3;
        /// User-mode accessible
        const U = 1 << 4;
        #[allow(missing_docs)]
        const G = 1 << 5;
        /// Accessed
        const A = 1 << 6;
        /// Dirty
        const D = 1 << 7;
        /// Copy On Write
        const C = 1 << 8;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageLevel {
    Huge = 0,
    Big = 1,
    Small = 2
}

impl PageLevel {
    pub const fn page_count(&self) -> usize {
        match self {
            PageLevel::Huge => 512 * 512,
            PageLevel::Big => 512,
            PageLevel::Small => 1,
        }
    }

    pub const fn lower(&self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Big,
            PageLevel::Big => PageLevel::Small,
            PageLevel::Small => PageLevel::Small,
        }
    }

    pub const fn higher(&self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Huge,
            PageLevel::Big => PageLevel::Huge,
            PageLevel::Small => PageLevel::Big,
        }
    }

    pub const fn lowest(&self) -> bool {
        match self {
            PageLevel::Small => true,
            _ => false
        }
    }

    pub const fn highest(&self) -> bool {
        match self {
            PageLevel::Huge => true,
            _ => false
        }
    }
}

impl From<usize> for PageLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Huge,
            1 => Self::Big,
            2 => Self::Small,
            _ => panic!("unsupport Page Level")
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
#[allow(missing_docs)]
/// page table entry structure
pub struct PageTableEntry {
    pub bits: usize,
}

#[allow(missing_docs)]
impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits((self.bits & ((1usize << 10) - 1)) as u16).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
    /// pte.is_leaf() == true, meaning this PTE points to the physical page, not to the next level of PTE.
    pub fn is_leaf(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty() && 
        (
            (self.flags() & PTEFlags::R) != PTEFlags::empty() ||
            (self.flags() & PTEFlags::W) != PTEFlags::empty() ||
            (self.flags() & PTEFlags::X) != PTEFlags::empty()
        )
    }
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.bits = ((self.bits >> 10) << 10) | flags.bits() as usize;
    }
}

/// page table structure
#[allow(missing_docs)]
pub struct PageTable {
    pub root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

/// Assume that it won't oom when creating/mapping.
#[allow(missing_docs)]
impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc_clean();
        let frame = frame.unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum, level: PageLevel) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.to_kern().get_pte_array()[*idx];
            if PageLevel::from(i) == level {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc_clean().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    #[deprecated]
    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.to_kern().get_pte_array()[*idx];
            if !pte.is_valid() {
                return None;
            }
            if i == 2 {
                result = Some(pte);
                break;
            }
            ppn = pte.ppn();
        }
        result
    }
    #[allow(unused)]
    pub fn find_leaf_pte(&self, vpn: VirtPageNum) -> Option<(&mut PageTableEntry, PageLevel)> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.to_kern().get_pte_array()[*idx];
            if !pte.is_valid() {
                return None;
            }
            if pte.is_leaf() || i == 2 {
                return Some((pte, i.into()));
            }
            ppn = pte.ppn();
        }
        None
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags, level: PageLevel) {
        match self.find_pte_create(vpn, level) {
            Some(pte) if !pte.is_valid() => {
                if level != PageLevel::Small {
                    //info!("[PageTable::map] mapping a {:?} page", level);
                }
                *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
            },
            _ => {
                panic!("vpn {:?} is mapped before mapping", vpn);
            }
        }
    }
    pub fn update_perm(&mut self, vpn: VirtPageNum, flags: PTEFlags) {
        match self.find_leaf_pte(vpn) {
            Some((pte, _)) if pte.is_valid() => {
                pte.set_flags(flags | PTEFlags::V);
            },
            _ => {
                panic!("vpn {:?} is invalid before updating", vpn);
            }
        }
    }
    pub fn try_map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags, level: PageLevel) -> Result<(), ()> {
        match self.find_pte_create(vpn, level) {
            Some(pte) if !pte.is_valid() => {
                *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
                Ok(())
            },
            _ => {
                Err(())
            }
        }
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        match self.find_leaf_pte(vpn) {
            Some((pte, _) ) if pte.is_valid()=> {
                *pte = PageTableEntry::empty();
            },
            _ => {
                panic!("vpn {:?} is invalid before unmapping", vpn);
            }
        }
    }
    pub fn try_unmap(&mut self, vpn: VirtPageNum) -> Result<(), ()> {
        match self.find_leaf_pte(vpn) {
            Some((pte, _) ) if pte.is_valid()=> {
                *pte = PageTableEntry::empty();
                Ok(())
            },
            _ => {
                Err(())
            }
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_leaf_pte(vpn).map(|(pte, _)| *pte)
    }
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_leaf_pte(va.floor()).map(|(pte, _)| {
            let aligned_pa: PhysAddr = pte.ppn().into();
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
    pub unsafe fn enable(&self) {
        // for x in self.root_ppn.get_pte_array() {
        //     info!("{:#x}", x.ppn().0 << 12);
        // }
        riscv::register::satp::write(self.token());
        Instruction::tlb_flush_all();
    }
}

/// translate a pointer to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn += 1;
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.to_kern().get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.to_kern().get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

/// Translate a pointer to a mutable u8 Vec end with `\0` through page table to a `String`
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table
            .translate_va(VirtAddr::from(va))
            .unwrap()
            .to_kern()
            .get_mut());
        if ch == 0 {
            break;
        }
        string.push(ch as char);
        va += 1;
    }
    string
}


#[allow(unused)]
///Translate a generic through page table and return a reference
pub fn translated_ref<T>(token: usize, ptr: *const T) -> &'static T {
    let page_table = PageTable::from_token(token);
    page_table
        .translate_va(VirtAddr::from(ptr as usize))
        .unwrap()
        .to_kern()
        .get_ref()
}
///Translate a generic through page table and return a mutable reference
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    page_table
        .translate_va(VirtAddr::from(va))
        .unwrap()
        .to_kern()
        .get_mut()
}


#[allow(unused)]
/// copy out 
pub fn copy_out<T: Copy>(page_table: &PageTable, mut dst: VirtAddr, mut src: &[T]) {
    let size = size_of::<T>();
    // size is power of 2 and less than PAGE_SIZE, dst is aligned to size
    assert!((size & (size - 1) == 0) && (size <= PAGE_SIZE) && (dst.0 & (size - 1) == 0));
    let mut bytes = src.len() * size;
    while bytes > 0 {
        let step = min(bytes, PAGE_SIZE - dst.page_offset());
        let len = step / size;
        let dst_ka = page_table.translate_va(dst).unwrap().to_kern();
        let dst_slice = unsafe {
            &mut *slice_from_raw_parts_mut(dst_ka.as_non_null_ptr().as_ptr(), len)
        };
        dst_slice.copy_from_slice(&src[..len]);
        src = &src[len..];
        dst += step;
        bytes -= step;
    }
}

#[allow(unused)]
/// copy out a str
pub fn copy_out_str(page_table: &PageTable, mut dst: VirtAddr, str: &str) {
    let mut src = str.as_bytes();
    let mut bytes = src.len() + 1;

    loop {
        let step = min(bytes, PAGE_SIZE - dst.page_offset());
        if step == bytes {
            break;
        }
        let dst_ka = page_table.translate_va(dst).unwrap().to_kern();
        let dst_slice = unsafe {
            &mut *slice_from_raw_parts_mut(dst_ka.as_non_null_ptr().as_ptr(), step)
        };
        dst_slice.copy_from_slice(&src[..step]);
        src = &src[step..];
        dst += step;
        bytes -= step;
    }

    let dst_ka = page_table.translate_va(dst).unwrap().to_kern();
    let dst_slice = unsafe {
        &mut *slice_from_raw_parts_mut(dst_ka.as_non_null_ptr().as_ptr(), bytes)
    };
    dst_slice[..bytes-1].copy_from_slice(&src[..bytes-1]);
    dst_slice[bytes-1] = 0;

}

#[allow(unused)]
/// copy in
pub fn copy_in<T: Copy>(page_table: &PageTable, mut dst: &mut [T], mut src: VirtAddr) {
    let size = size_of::<T>();
    // size is power of 2 and less than PAGE_SIZE, dst is aligned to size
    assert!((size & (size - 1) == 0) && (size <= PAGE_SIZE) && (src.0 & (size - 1) == 0));
    let mut bytes = dst.len() * size;
    while bytes > 0 {
        let step = min(bytes, PAGE_SIZE - src.page_offset());
        let len = step / size;
        let src_ka = page_table.translate_va(src).unwrap().to_kern();
        let src_slice = unsafe {
            &mut *slice_from_raw_parts_mut(src_ka.as_non_null_ptr().as_ptr(), len)
        };
        dst[..len].copy_from_slice(src_slice);
        dst = &mut dst[len..];
        src += step;
        bytes -= step;
    }
}

#[allow(unused)]
/// copy in a str
pub unsafe fn copy_in_str(page_table: &PageTable, mut str: &mut str, mut src: VirtAddr) {
    let mut dst = str.as_bytes_mut();
    copy_in(page_table, dst, src);
}

///Array of u8 slice that user communicate with os
pub struct UserBuffer {
    ///U8 vec
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    ///Create a `UserBuffer` by parameter
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    ///Length of `UserBuffer`
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}

impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffers,
            current_buffer: 0,
            current_idx: 0,
        }
    }
}
/// Iterator of `UserBuffer`
pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}