//! Page for page cache

use core::{cmp, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

use alloc::sync::{Arc, Weak};
use hal::{addr::{PhysPageNum, RangePPNHal}, allocator::FrameAllocatorHal, util::smart_point::StrongArc};

use crate::{fs::vfs::Inode, mm::{allocator::{frames_alloc, FrameAllocator, SlabAllocator}, FrameTracker}};

pub struct Page {
    /// page frame state or attribute
    pub is_dirty: AtomicBool,
    /// offset in a file (if is owned by file)
    pub index: usize, 
    /// the physical frame it owns
    pub frame: StrongArc<FrameTracker, SlabAllocator>,
}

unsafe impl Send for Page {}
unsafe impl Sync for Page {}

pub const PAGE_SIZE: usize = 4096;
impl Page {
    /// create a Page by allocating a frame
    pub fn new(index: usize) -> Arc<Self> {
        let frame = FrameAllocator.alloc_tracker(1).expect("[Page]: allocating page failed");
        // clean up the page
        frame.range_ppn.get_slice_mut::<u8>().fill(0);
        Arc::new(Self {
            is_dirty: AtomicBool::new(false), // need more flags
            index: index,
            frame: StrongArc::new_in(frame, SlabAllocator),
        })
    }
    /// return the mutable slice of the raw data the page points to
    pub fn get_slice_mut<T>(&mut self) -> &mut [T] {
        self.frame.range_ppn.get_slice_mut::<T>()
    }
    /// return the immutable slice of the raw data the page points to
    pub fn get_slice<T>(&self) -> &[T] {
        self.frame.range_ppn.get_slice::<T>()
    }
    /// return the physical page number of this page
    pub fn ppn(&self) -> PhysPageNum {
        self.frame.range_ppn.start
    }
    /// return the physical frame
    pub fn frame(&self) -> StrongArc<FrameTracker, SlabAllocator> {
        self.frame.clone()
    }
    /// write the page at a specific offset
    /// this only will only be call when user try to write cached page
    /// so page should be set dirty and wait for Inode to flush itself
    /// as the page dont hold any info about the inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        assert!(offset < PAGE_SIZE);
        let write_size = cmp::min(PAGE_SIZE - offset, buf.len());
        let page_slice = self.frame.range_ppn.get_slice_mut::<u8>();
        page_slice[offset..offset + write_size].copy_from_slice(&buf[..write_size]);
        write_size
    }
    /// read out the page at a specific offset
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        assert!(offset < PAGE_SIZE);
        let read_size = cmp::min(PAGE_SIZE - offset, buf.len());
        let page_slice = self.frame.range_ppn.get_slice::<u8>();
        buf[..read_size].copy_from_slice(&page_slice[offset..offset + read_size]);
        read_size
    }
    /// read from given Inode and the offset in Inode
    /// we assert that the offset should be page-aligned
    /// load the inode data into the page
    pub fn read_from(&mut self, inode: Arc<dyn Inode>, offset: usize) -> usize {
        assert!(offset % PAGE_SIZE == 0);
        let page_slice = self.frame.range_ppn.get_slice_mut::<u8>();
        let read_size = inode.read_at(offset, page_slice).expect("inode read failed");
        read_size
    }
    /// write to given Inode and the offset in Inode
    /// we assert that the offset should be page-aligned
    /// should only write back if Page is dirty
    pub fn write_back(&self, inode: Arc<dyn Inode>, offset: usize) -> usize {
        assert!(offset % PAGE_SIZE == 0);
        assert!(self.is_dirty() == true);
        let page_slice = self.frame.range_ppn.get_slice::<u8>();
        let write_size = inode.write_at(offset, page_slice).expect("inode write failed");
        // no need to care about the EOF, write_at will handle this
        write_size
    }
    /// set the page dirty
    pub fn set_dirty(&self) {
        self.is_dirty.store(true, Ordering::Release);
    }
    /// is the page dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty.load(Ordering::Acquire)
    }
}