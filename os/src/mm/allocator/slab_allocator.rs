use core::{alloc::GlobalAlloc, marker::PhantomPinned, mem, pin::Pin, ptr::{null_mut, slice_from_raw_parts_mut, NonNull}, usize};

use alloc::{alloc::{AllocError, Allocator, Global}, boxed::Box, collections::btree_map::BTreeMap, format, string::ToString};
use fatfs::info;
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal}, allocator::FrameAllocatorHal, constant::{Constant, ConstantsHal}, println, util::mutex::Mutex};
use range_map::RangeMap;

use crate::{mm::allocator::next_power_of_two, sync::mutex::{spin_mutex::SpinMutex, Spin, SpinNoIrqLock}};

use super::{FrameAllocator, HeapAllocator};

#[global_allocator]
static SLAB_ALLOCATOR: SlabAllocator = SlabAllocator;

/// slab allocator
#[allow(unused)]
pub static SLAB_ALLOCATOR_INNER: SlabAllocatorInner = SlabAllocatorInner::new();

/// Slab Allocator
#[derive(Debug, Clone)]
pub struct SlabAllocator;


unsafe impl Allocator for SlabAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        if SlabAllocatorInner::check_layout(layout) {
            Ok(SLAB_ALLOCATOR_INNER.alloc_by_layout(layout).map(
                |ptr| {
                    NonNull::slice_from_raw_parts(ptr, layout.size())
                }
            ).ok_or(AllocError)?)
        } else {
            FrameAllocator.allocate(layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        if SlabAllocatorInner::check_layout(layout) {
            SLAB_ALLOCATOR_INNER.dealloc_by_layout(ptr, layout);
        } else {
            FrameAllocator.deallocate(ptr, layout);
        }
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut times = 2;
        loop {
            if let Ok(mut ptr) = Allocator::allocate(self, layout) {
                let ret = &mut ptr.as_mut()[0] as *mut u8;
                // log::info!("[GlobalAlloc] alloc: ptr: {:#x} layout: {:?}", ret as usize, layout);
                return ret;
            } else {
                log::warn!("failed alloc layout: {:?}", layout);
                SLAB_ALLOCATOR_INNER.shrink();
                SLAB_BLOCK_SLAB_CACHE.lock().shrink();
            }
            times -= 1;
            if times == 0 {
                break;
            }
        }
        // SLAB_ALLOCATOR_INNER.info();
        // println!("slab block slab cache:");
        // SLAB_BLOCK_SLAB_CACHE.lock().info();
        return 0 as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // log::info!("[GlobalAlloc] dealloc: ptr: {:#x} layout: {:?}", ptr as usize, layout);
        if let Some(ptr) = NonNull::new(ptr) {
            Allocator::deallocate(&self, ptr, layout);
        }
    }
}

/// Slab Allocator's Inner
pub struct SlabAllocatorInner {
    pub cache8: SpinNoIrqLock<SmallSlabCache<8>>, 
    pub cache16: SpinNoIrqLock<SmallSlabCache<16>>, 
    pub cache32: SpinNoIrqLock<SmallSlabCache<32>>, 
    pub cache64: SpinNoIrqLock<SmallSlabCache<64>>, 
    pub cache96: SpinNoIrqLock<SmallSlabCache<96>>,
    pub cache128: SpinNoIrqLock<SmallSlabCache<128>>, 
    pub cache192: SpinNoIrqLock<SlabCache<192>>, 
    pub cache256: SpinNoIrqLock<SlabCache<256>>, 
    pub cache512: SpinNoIrqLock<SlabCache<512>>, 
    pub cache1024: SpinNoIrqLock<SlabCache<1024>>,
    pub cache2048: SpinNoIrqLock<SlabCache<2048>>, 
    pub cache4096: SpinNoIrqLock<SlabCache<4096>>, 
    pub cache8192: SpinNoIrqLock<SlabCache<8192>>, 
}

unsafe impl Sync for SlabAllocatorInner {}

#[allow(unused)]
impl SlabAllocatorInner {
    /// new
    pub const fn new() -> Self {
        Self {
            cache8: SpinNoIrqLock::new(SmallSlabCache::<8>::new()),
            cache16: SpinNoIrqLock::new(SmallSlabCache::<16>::new()),
            cache32: SpinNoIrqLock::new(SmallSlabCache::<32>::new()),
            cache64: SpinNoIrqLock::new(SmallSlabCache::<64>::new()),
            cache96: SpinNoIrqLock::new(SmallSlabCache::<96>::new()),
            cache128: SpinNoIrqLock::new(SmallSlabCache::<128>::new()),
            cache192: SpinNoIrqLock::new(SlabCache::<192>::new()),
            cache256: SpinNoIrqLock::new(SlabCache::<256>::new()),
            cache512: SpinNoIrqLock::new(SlabCache::<512>::new()),
            cache1024: SpinNoIrqLock::new(SlabCache::<1024>::new()),
            cache2048: SpinNoIrqLock::new(SlabCache::<2048>::new()),
            cache4096: SpinNoIrqLock::new(SlabCache::<4096>::new()),
            cache8192: SpinNoIrqLock::new(SlabCache::<8192>::new()),
        }
    }

    pub fn check_layout(layout: core::alloc::Layout) -> bool{
        layout.size() <= 8192 && layout.align() <= layout.size() && layout.align() <= 4096
    }

    /// release useless frames
    pub fn shrink(&self) {
        self.cache8.lock().shrink();
        self.cache16.lock().shrink();
        self.cache32.lock().shrink();
        self.cache64.lock().shrink();
        self.cache96.lock().shrink();
        self.cache128.lock().shrink();
        self.cache192.lock().shrink();
        self.cache256.lock().shrink();
        self.cache512.lock().shrink();
        self.cache1024.lock().shrink();
        self.cache2048.lock().shrink();
        self.cache4096.lock().shrink();
        self.cache8192.lock().shrink();
    }

    pub fn alloc_by_layout(&self, layout: core::alloc::Layout) -> Option<NonNull<u8>> {
        match layout.pad_to_align().size() {
            0..=8 => {
                self.cache8.lock().alloc()
            },
            9..=16 => {
                self.cache16.lock().alloc()
            },
            17..=32 => {
                self.cache32.lock().alloc()
            },
            33..=64 => {
                self.cache64.lock().alloc()
            },
            65..=96 => {
                self.cache96.lock().alloc()
            },
            97..=128 => {
                self.cache128.lock().alloc()
            },
            129..=192 => {
                self.cache192.lock().alloc()
            },
            193..=256 => {
                self.cache256.lock().alloc()
            },
            257..=512 => {
                self.cache512.lock().alloc()
            },
            513..=1024 => {
                self.cache1024.lock().alloc()
            },
            1025..=2048 => {
                self.cache2048.lock().alloc()
            },
            2049..=4096 => {
                self.cache4096.lock().alloc()
            },
            4097..=8192 => {
                let ptr = self.cache8192.lock().alloc();
                // if let Some(ptr) = ptr {
                //     log::info!("alloc ptr: {:#x} layout: {:?}", ptr.as_ptr() as usize, layout);
                // }
                ptr
            },
            _ => None
        }
    }

    /// alloc a payload
    pub fn alloc<T: Sized>(&self) -> Option<NonNull<T>> {
        self.alloc_by_layout(core::alloc::Layout::new::<T>()).map(|ptr| ptr.cast())
    }

    pub fn dealloc_by_layout(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        match layout.pad_to_align().size() {
            0..=8 => {
                self.cache8.lock().dealloc(ptr);
            },
            9..=16 => {
                self.cache16.lock().dealloc(ptr);
            },
            17..=32 => {
                self.cache32.lock().dealloc(ptr);
            },
            33..=64 => {
                self.cache64.lock().dealloc(ptr);
            },
            65..=96 => {
                self.cache96.lock().dealloc(ptr);
            },
            97..=128 => {
                self.cache128.lock().dealloc(ptr);
            },
            129..=192 => {
                self.cache192.lock().dealloc(ptr);
            },
            193..=256 => {
                self.cache256.lock().dealloc(ptr);
            },
            257..=512 => {
                self.cache512.lock().dealloc(ptr);
            },
            513..=1024 => {
                self.cache1024.lock().dealloc(ptr);
            },
            1025..=2048 => {
                self.cache2048.lock().dealloc(ptr);
            },
            2049..=4096 => {
                self.cache4096.lock().dealloc(ptr);
            },
            4097..=8192 => {
                // log::info!("dealloc ptr: {:#x} layout: {:?}", ptr.as_ptr() as usize, layout);
                self.cache8192.lock().dealloc(ptr);
            },
            _ => {}
        }
    }

    /// dealloc a payload
    pub fn dealloc<T: Sized>(&self, ptr: NonNull<T>) {
        self.dealloc_by_layout(ptr.cast(), core::alloc::Layout::new::<T>());
    }

    pub fn info(&self) {
        println!("cache8:");
        self.cache8.lock().info();
        println!("cache16:");
        self.cache16.lock().info();
        println!("cache32:");
        self.cache32.lock().info();
        println!("cache64:");
        self.cache64.lock().info();
        println!("cache96:");
        self.cache96.lock().info();
        println!("cache128:");
        self.cache128.lock().info();
        println!("cache192:");
        self.cache192.lock().info();
        println!("cache256:");
        self.cache256.lock().info();
        println!("cache512:");
        self.cache512.lock().info();
        println!("cache1024:");
        self.cache1024.lock().info();
        println!("cache2048:");
        self.cache2048.lock().info();
        println!("cache4096:");
        self.cache4096.lock().info();
        println!("cache8192:");
        self.cache8192.lock().info();
    }
}

#[allow(missing_docs)]
pub union FreeNode<const S: usize> {
    next: *mut FreeNode<S>,
    _data: [u8; S],
}

#[repr(C)]
#[derive(Debug)]
/// slab block for big object
struct SlabBlock<const S: usize> {
    /// last block
    last: *mut SlabBlock<S>,
    /// next block
    next: *mut SlabBlock<S>,
    /// size
    size: usize,
    /// node list head
    head: *mut FreeNode<S>
}

unsafe impl<const S: usize> Send for SlabBlock<S> {}

#[allow(unused, missing_docs)]
impl<const S: usize> SlabBlock<S> {
    pub fn page_cnt() -> usize {
        super::next_power_of_two((S << 3) >> Constant::PAGE_SIZE_BITS)
    }

    pub fn cap() -> usize {
        (Self::page_cnt() << Constant::PAGE_SIZE_BITS) / S
    }

    pub fn floor(mut addr: usize) -> PhysPageNum {
        addr &= !Constant::KERNEL_ADDR_SPACE.start;
        addr &= !(Constant::PAGE_SIZE-1);
        addr >>= Constant::PAGE_SIZE_BITS;
        PhysPageNum(addr)
    }

    fn dealloc(&mut self) {
        let start_ppn = Self::floor((self.head as usize).into());
        let end_ppn = start_ppn + Self::page_cnt();
        log::info!("[SlabBlock::dealloc] {:#x} {:#x}", start_ppn.start_addr().0, end_ppn.start_addr().0);
        FrameAllocator.dealloc(start_ppn..end_ppn);
    }
}

impl<const S: usize> LinkedNode for SlabBlock<S> {
    fn last(&mut self) -> &mut *mut Self {
        &mut self.last
    }

    fn next(&mut self) -> &mut *mut Self {
        &mut self.next
    }
}


/// slab cache for big object
pub struct SlabCache<const S: usize> {
    blocks: RangeMap<PhysPageNum, Box<SlabBlock<S>, SlabBlockSlabAllocator>, HeapAllocator>,
    empty_blk_list: LinkedStack<SlabBlock<S>>,
    free_blk_list: LinkedStack<SlabBlock<S>>,
    full_blk_list: LinkedStack<SlabBlock<S>>,
}

#[allow(unused, missing_docs)]
impl<const S: usize> SlabCache<S> {
    pub const fn new() -> Self {
        Self {
            blocks: RangeMap::new_in(HeapAllocator),
            empty_blk_list: LinkedStack::new(),
            free_blk_list: LinkedStack::new(),
            full_blk_list: LinkedStack::new(),
        }
    }

    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        if self.free_blk_list.is_empty() {
            if let Some(t) = self.empty_blk_list.pop() {
                self.free_blk_list.push(t);
            } else {
                let frames = FrameAllocator.alloc_with_align(
                    SlabBlock::<S>::page_cnt(), 
                    0
                )?;
                let free_nodes_ptr = frames.start.start_addr().get_ptr::<FreeNode<S>>();

                let blk = Box::new_in(
                    SlabBlock::<S> {
                        last: null_mut(),
                        next: null_mut(),
                        size: 0,
                        head: free_nodes_ptr
                    }, 
                    SlabBlockSlabAllocator
                );

                self.blocks.try_insert(frames.clone(), blk).unwrap();
                let blk = self.blocks.get_mut(frames.start).unwrap();
                let free_nodes = unsafe {
                    &mut *slice_from_raw_parts_mut(free_nodes_ptr, SlabBlock::<S>::cap())
                };

                let mut last = unsafe { &mut *free_nodes_ptr };
                for node in free_nodes[1..].iter_mut() {
                    last.next = node;
                    last = node;
                }
                last.next = null_mut();
                self.free_blk_list.push(blk);
            }
        }

        let blk = unsafe { &mut *self.free_blk_list.head };
        if blk.head.is_null() {
            panic!("SlabBlock head is null");
        }
        let ret = blk.head;
        unsafe {
            blk.head = (*blk.head).next;
            (*ret).next = 0 as _;
        }
        blk.size += 1;
        if blk.head.is_null() {
            self.free_blk_list.pop();
            self.full_blk_list.push(blk);
        }
        NonNull::new(ret as *mut u8)
    }

    pub fn dealloc(&mut self, ptr: NonNull<u8>) -> Option<()> {
        let mut ptr: NonNull<FreeNode<S>> = ptr.cast();
        let addr = ptr.addr().get();
        let ppn = SlabBlock::<S>::floor(addr);
        let blk = self.blocks.get_mut(ppn).unwrap();
        let free_node = unsafe { ptr.as_mut() };
        free_node.next = blk.head;
        blk.head = free_node;

        if blk.size == SlabBlock::<S>::cap() {
            self.full_blk_list.remove(blk);
            self.free_blk_list.push(blk);
        } else if blk.size == 1 {
            self.free_blk_list.remove(blk);
            self.empty_blk_list.push(blk);
        }
        blk.size -= 1;
        Some(())
    }

    pub fn shrink(&mut self) {
        let mut blk_ptr = self.empty_blk_list.head;
        self.empty_blk_list.head = null_mut();
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            blk.dealloc();
            let ppn = SlabBlock::<S>::floor(blk.head as usize);
            let (range, _) = self.blocks.get_key_value(ppn).unwrap();
            self.blocks.force_remove_one(range);
            blk_ptr = next;
        };
    }

    pub fn info(&mut self) {
        println!("SlabCache {:#x}", self as *const _ as usize);
        println!("block cap: {},block page count: {}", SlabBlock::<S>::cap(), SlabBlock::<S>::page_cnt());
        let mut blk_ptr = self.empty_blk_list.head;
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            println!("Empty Block {:?}", blk);
            blk_ptr = next;
        };
        blk_ptr = self.free_blk_list.head;
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            println!("Free Block {:?}", blk);
            blk_ptr = next;
        };
        blk_ptr = self.full_blk_list.head;
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            println!("Full Block {:?}", blk);
            blk_ptr = next;
        };
    }
}

/// linked node
#[allow(unused, missing_docs)]
trait LinkedNode {
    fn last(&mut self) -> &mut *mut Self;
    fn next(&mut self) -> &mut *mut Self;
}

/// linked stack
#[allow(unused, missing_docs)]
struct LinkedStack<T: LinkedNode> {
    head: *mut T,
}

unsafe impl<T: LinkedNode> Send for LinkedStack<T> {}

#[allow(unused, missing_docs)]
impl<T: LinkedNode> LinkedStack<T> {

    const fn new() -> Self {
        Self { head: null_mut() }
    }

    fn push(&mut self, val: &mut T) {
        *val.next() = self.head;
        *val.last() = null_mut();
        if !self.head.is_null() {
            unsafe { *(*self.head).last() = val };
        }
        self.head = val;
    }

    fn pop(&mut self) -> Option<&mut T> {
        if self.head.is_null() {
            return None;
        }
        let ret = self.head;
        self.head = unsafe { *(*self.head).next() };
        if !self.head.is_null() {
            unsafe { *(*self.head).last() = null_mut() };
        }
        let ret = unsafe { &mut *ret };
        *ret.last() = null_mut();
        *ret.next() = null_mut();
        Some(ret)
    }

    fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    fn remove(&mut self, t: &mut T) {
        if t.last().is_null() {
            assert!(self.head == t);
            self.head = *t.next();
            if !self.head.is_null() {
                unsafe { *(*self.head).last() = null_mut() };
            }
        } else {
            assert!(self.head != t);
            unsafe { *(**t.last()).next() = *t.next() };
            if !t.next().is_null() {
                unsafe { *(**t.next()).last() = *t.last() };
            }
        }
        *t.last() = null_mut();
        *t.next() = null_mut();
    }
}

#[allow(missing_docs)]
const CACHE_SIZE: usize = size_of::<SlabBlock<0>>();

#[allow(missing_docs)]
static SLAB_BLOCK_SLAB_CACHE: SpinNoIrqLock<SmallSlabCache<CACHE_SIZE>> = SpinNoIrqLock::new(SmallSlabCache::new());

#[allow(missing_docs)]
struct SlabBlockSlabAllocator;

unsafe impl Allocator for SlabBlockSlabAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        assert_eq!(layout, core::alloc::Layout::from_size_align(size_of::<SlabBlock<0>>(), align_of::<SlabBlock<0>>()).unwrap());
        SLAB_BLOCK_SLAB_CACHE.lock().alloc().map(|ptr| {
            NonNull::slice_from_raw_parts(ptr, layout.size())
        }).ok_or(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        assert_eq!(layout, core::alloc::Layout::from_size_align(size_of::<SlabBlock<0>>(), align_of::<SlabBlock<0>>()).unwrap());
        SLAB_BLOCK_SLAB_CACHE.lock().dealloc(ptr);
    }
}

#[repr(C)]
#[derive(Debug)]
/// slab block for small object
struct SmallSlabBlock<const S: usize> {
    /// owner
    owner: *const SmallSlabCache<S>,
    /// last block
    last: *mut SmallSlabBlock<S>,
    /// next block
    next: *mut SmallSlabBlock<S>,
    /// size
    size: usize,
    /// node list head
    head: *mut FreeNode<S>
}

unsafe impl<const S: usize> Send for SmallSlabBlock<S> {}

#[allow(unused, missing_docs)]
impl<const S: usize> SmallSlabBlock<S> {
    pub fn page_cnt() -> usize {
        1
    }

    pub unsafe fn free_nodes_ptr(&self) -> *mut FreeNode<S> {
        let blk_ptr = self as *const Self;
        let free_nodes_ptr = blk_ptr.byte_add(core::cmp::max(size_of::<SmallSlabBlock<S>>(), next_power_of_two(S))) as *mut FreeNode<S>;
        free_nodes_ptr
    }

    pub fn cap() -> usize {
        (Constant::PAGE_SIZE - core::cmp::max(size_of::<SmallSlabBlock<S>>(), next_power_of_two(S))) / S
    }

    pub fn floor(mut addr: usize) -> PhysPageNum {
        addr &= !Constant::KERNEL_ADDR_SPACE.start;
        addr &= !(Constant::PAGE_SIZE-1);
        addr >>= Constant::PAGE_SIZE_BITS; 
        PhysPageNum(addr)
    }

    fn dealloc(&mut self) {
        let start_ppn = Self::floor(self as *mut _ as usize);
        let end_ppn = start_ppn + 1;
        log::info!("[SmallSlabBlock::dealloc] {:#x} {:#x}", start_ppn.start_addr().0, end_ppn.start_addr().0);
        FrameAllocator.dealloc(start_ppn..end_ppn);
    }
}

impl<const S: usize> LinkedNode for SmallSlabBlock<S> {
    fn last(&mut self) -> &mut *mut Self {
        &mut self.last
    }

    fn next(&mut self) -> &mut *mut Self {
        &mut self.next
    }
}

/// slab cache for small object
pub struct SmallSlabCache<const S: usize> {
    empty_blk_list: LinkedStack<SmallSlabBlock<S>>,
    free_blk_list: LinkedStack<SmallSlabBlock<S>>,
    full_blk_list: LinkedStack<SmallSlabBlock<S>>,
    _pinned_marker: PhantomPinned,
}


#[allow(unused, missing_docs)]
impl<const S: usize> SmallSlabCache<S> {
    pub const fn new() -> Self {
        Self {
            empty_blk_list: LinkedStack::new(),
            free_blk_list: LinkedStack::new(),
            full_blk_list: LinkedStack::new(),
            _pinned_marker: PhantomPinned
        }
    }

    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        if self.free_blk_list.is_empty() {
            if let Some(t) = self.empty_blk_list.pop() {
                self.free_blk_list.push(t);
            } else {
                let frames = FrameAllocator.alloc_with_align(
                    SlabBlock::<S>::page_cnt(), 
                    0
                )?;
                let blk_ptr = frames.start.start_addr().get_ptr::<SmallSlabBlock<S>>();
                let blk = unsafe { &mut *blk_ptr };
                blk.owner = self;
                blk.size = 0;
                let free_nodes_ptr = unsafe {
                    blk.free_nodes_ptr()
                };
                let free_nodes = unsafe {
                    &mut *slice_from_raw_parts_mut(free_nodes_ptr, SmallSlabBlock::<S>::cap())
                };
                blk.head = free_nodes_ptr;
                let mut last = unsafe { &mut *free_nodes_ptr };
                for node in free_nodes[1..].iter_mut() {
                    last.next = node;
                    last = node;
                }
                last.next = null_mut();
                self.free_blk_list.push(blk);
            }
        }

        let blk = unsafe { &mut *self.free_blk_list.head };
        if blk.head.is_null() {
            panic!("SlabBlock head is null");
        }
        let ret = blk.head;
        unsafe {
            blk.head = (*blk.head).next;
            (*ret).next = 0 as _;
        }
        blk.size += 1;
        if blk.head.is_null() {
            self.free_blk_list.pop();
            self.full_blk_list.push(blk);
        }
        NonNull::new(ret as *mut u8)
    }

    pub fn dealloc(&mut self, ptr: NonNull<u8>) -> Option<()> {
        let mut ptr: NonNull<FreeNode<S>> = ptr.cast();
        let addr = ptr.addr().get();
        let ppn = SlabBlock::<S>::floor(addr);
        let blk = ppn.start_addr().get_mut::<SmallSlabBlock<S>>();
        if blk.owner != self {
            panic!("block {:?} is not belong to this cache {:#x}", blk, self as *const _ as usize);
        }
        let free_node = unsafe { ptr.as_mut() };
        free_node.next = blk.head;
        blk.head = free_node;

        if blk.size == SmallSlabBlock::<S>::cap() {
            self.full_blk_list.remove(blk);
            self.free_blk_list.push(blk);
        } else if blk.size == 1 {
            self.free_blk_list.remove(blk);
            self.empty_blk_list.push(blk);
        }
        blk.size -= 1;
        Some(())
    }

    pub fn shrink(&mut self) {
        let mut blk_ptr = self.empty_blk_list.head;
        self.empty_blk_list.head = null_mut();
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            blk.dealloc();
            blk_ptr = next;
        };
    }

    pub fn info(&mut self) {
        println!("SmallSlabCache {:#x}", self as *const _ as usize);
        println!("block cap: {},block page count: {}", SmallSlabBlock::<S>::cap(), SmallSlabBlock::<S>::page_cnt());
        let mut blk_ptr = self.empty_blk_list.head;
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            println!("Empty Block {:?}", blk);
            blk_ptr = next;
        };
        blk_ptr = self.free_blk_list.head;
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            println!("Free Block {:?}", blk);
            blk_ptr = next;
        };
        blk_ptr = self.full_blk_list.head;
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            println!("Full Block {:?}", blk);
            blk_ptr = next;
        };
    }
}