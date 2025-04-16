use core::{marker::PhantomPinned, mem, ptr::{null_mut, slice_from_raw_parts_mut, NonNull}};

use alloc::{alloc::{AllocError, Allocator, Global}, collections::btree_map::BTreeMap};
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal}, allocator::FrameAllocatorHal, constant::{Constant, ConstantsHal}, println, util::mutex::Mutex};

use crate::sync::mutex::{spin_mutex::SpinMutex, Spin};

use super::FrameAllocator;

/// slab allocator
#[allow(unused)]
pub static SLAB_ALLOCATOR_INNER: SlabAllocatorInner = SlabAllocatorInner::new();

/// Slab Allocator's Inner
pub struct SlabAllocatorInner {
    pub cache8: SpinMutex<SlabCache<8>, Spin>, 
    pub cache16: SpinMutex<SlabCache<16>, Spin>, 
    pub cache32: SpinMutex<SlabCache<32>, Spin>, 
    pub cache64: SpinMutex<SlabCache<64>, Spin>, 
    pub cache96: SpinMutex<SlabCache<96>, Spin>,
    pub cache128: SpinMutex<SlabCache<128>, Spin>, 
    pub cache192: SpinMutex<SlabCache<192>, Spin>, 
    pub cache256: SpinMutex<SlabCache<256>, Spin>, 
    pub cache512: SpinMutex<SlabCache<512>, Spin>, 
    pub cache1024: SpinMutex<SlabCache<1024>, Spin>,
    pub cache2048: SpinMutex<SlabCache<2048>, Spin>, 
    pub cache4096: SpinMutex<SlabCache<4096>, Spin>, 
    pub cache8192: SpinMutex<SlabCache<8192>, Spin>, 
}

unsafe impl Sync for SlabAllocatorInner {}

/// Slab Allocator
#[derive(Clone)]
pub struct SlabAllocator;


unsafe impl Allocator for SlabAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        Ok(SLAB_ALLOCATOR_INNER.alloc_by_layout(layout).map(
            |ptr| {
                NonNull::slice_from_raw_parts(ptr, layout.size())
            }
        ).ok_or(AllocError)?)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        SLAB_ALLOCATOR_INNER.dealloc_by_layout(ptr, layout);
    }
}

#[allow(unused)]
impl SlabAllocatorInner {
    /// new
    pub const fn new() -> Self {
        Self {
            cache8: SpinMutex::new(SlabCache::<8>::new()),
            cache16: SpinMutex::new(SlabCache::<16>::new()),
            cache32: SpinMutex::new(SlabCache::<32>::new()),
            cache64: SpinMutex::new(SlabCache::<64>::new()),
            cache96: SpinMutex::new(SlabCache::<96>::new()),
            cache128: SpinMutex::new(SlabCache::<128>::new()),
            cache192: SpinMutex::new(SlabCache::<192>::new()),
            cache256: SpinMutex::new(SlabCache::<256>::new()),
            cache512: SpinMutex::new(SlabCache::<512>::new()),
            cache1024: SpinMutex::new(SlabCache::<1024>::new()),
            cache2048: SpinMutex::new(SlabCache::<2048>::new()),
            cache4096: SpinMutex::new(SlabCache::<4096>::new()),
            cache8192: SpinMutex::new(SlabCache::<8192>::new()),
        }
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
                self.cache8192.lock().alloc()
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
                self.cache8192.lock().dealloc(ptr);
            },
            _ => {}
        }
    }

    /// dealloc a payload
    pub fn dealloc<T: Sized>(&self, ptr: NonNull<T>) {
        self.dealloc_by_layout(ptr.cast(), core::alloc::Layout::new::<T>());
    }
}

#[allow(missing_docs)]
pub union FreeNode<const S: usize> {
    next: *mut FreeNode<S>,
    _data: [u8; S],
}

#[repr(C)]
#[allow(missing_docs)]
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

#[allow(unused, missing_docs)]
impl<const S: usize> SlabBlock<S> {
    pub fn page_cnt() -> usize {
        super::next_power_of_two((S << 3) >> Constant::PAGE_SIZE_BITS)
    }

    pub fn cap() -> usize {
        (Self::page_cnt() << Constant::PAGE_SIZE_BITS) / S
    }

    pub fn floor(addr: PhysAddr) -> PhysPageNum {
        PhysAddr(addr.0 & !((Self::page_cnt() << Constant::PAGE_SIZE_BITS)-1)).floor()
    }

    fn dealloc(&mut self) {
        let start_ppn = Self::floor((self.head as usize).into());
        let end_ppn = start_ppn + Self::page_cnt();
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

#[allow(unused, missing_docs)]
pub struct SlabCache<const S: usize> {
    blocks: BTreeMap<PhysPageNum, SlabBlock<S>>,
    empty_blk_list: LinkedStack<SlabBlock<S>>,
    free_blk_list: LinkedStack<SlabBlock<S>>,
    full_blk_list: LinkedStack<SlabBlock<S>>,
    _marker: PhantomPinned,
}

#[allow(unused, missing_docs)]
impl<const S: usize> SlabCache<S> {
    pub const fn new() -> Self {
        Self {
            blocks: BTreeMap::new(),
            empty_blk_list: LinkedStack::new(),
            free_blk_list: LinkedStack::new(),
            full_blk_list: LinkedStack::new(),
            _marker: PhantomPinned,
        }
    }

    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        loop {
            if self.free_blk_list.is_empty() {
                if let Some(t) = self.empty_blk_list.pop() {
                    self.free_blk_list.push(t);
                    continue;
                }
                let frames = FrameAllocator.alloc_with_align(
                    SlabBlock::<S>::page_cnt(), 
                    super::log2(SlabBlock::<S>::page_cnt())
                )?;
                let free_nodes_ptr = frames.start.start_addr().get_ptr::<FreeNode<S>>();

                let blk = SlabBlock::<S> {
                    last: null_mut(),
                    next: null_mut(),
                    size: 0,
                    head: free_nodes_ptr,
                };

                self.blocks.insert(frames.start, blk);
                let blk = self.blocks.get_mut(&frames.start).unwrap();

                let free_nodes = unsafe {
                    &mut *slice_from_raw_parts_mut(free_nodes_ptr, SlabBlock::<S>::cap())
                };
                blk.head = free_nodes_ptr;
                let mut last = unsafe { &mut *free_nodes_ptr };
                for node in free_nodes[1..].iter_mut() {
                    last.next = node;
                    last = node;
                }
                last.next = null_mut();
                self.free_blk_list.push(blk);
                continue;
            }

            let blk = unsafe { &mut *self.free_blk_list.head };
            if blk.head.is_null() {
                self.free_blk_list.pop();
                self.full_blk_list.push(blk);
                continue;
            }
            let ret = blk.head;
            unsafe {
                blk.head = (*blk.head).next;
                (*ret).next = 0 as _;
            }
            blk.size += 1;
            break NonNull::new(ret as *mut u8);
        }
    }

    pub fn dealloc(&mut self, ptr: NonNull<u8>) -> Option<()> {
        let mut ptr: NonNull<FreeNode<S>> = ptr.cast();
        let addr = ptr.addr().get();
        let ppn = SlabBlock::<S>::floor(addr.into());
        let blk = self.blocks.get_mut(&ppn)?;

        let free_node = unsafe { ptr.as_mut() };
        free_node.next = blk.head;
        blk.head = free_node;

        if blk.size == SlabBlock::<S>::cap() {
            blk.size -= 1;
            self.full_blk_list.remove(blk);
            self.free_blk_list.push(blk);
        } else if blk.size == 1 {
            blk.size -= 1;
            self.free_blk_list.remove(blk);
            self.empty_blk_list.push(blk);
        }
        Some(())
    }

    pub fn shrink(&mut self) {
        let mut blk_ptr = self.free_blk_list.head;
        while !blk_ptr.is_null() {
            let blk = unsafe {&mut *blk_ptr};
            let next = blk.next;
            blk.dealloc();
            let ppn = SlabBlock::<S>::floor((blk.head as usize).into());
            self.blocks.remove(&ppn).unwrap();
            blk_ptr = next;
        };
        self.free_blk_list.head = null_mut();
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

#[allow(unused, missing_docs)]
impl<T: LinkedNode> LinkedStack<T> {

    const fn new() -> Self {
        Self { head: null_mut() }
    }

    fn push(&mut self, val: &mut T) {
        *val.next() = self.head;
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
        Some(unsafe { &mut *ret })
    }

    fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    fn remove(&mut self, t: &mut T) {
        if t.last().is_null() {
            debug_assert!(self.head == t);
            self.head = *t.next();
            if !self.head.is_null() {
                unsafe { *(*self.head).last() = null_mut() };
            }
        } else {
            debug_assert!(self.head != t);
            unsafe { *(**t.last()).next() = *t.next() };
            if !t.next().is_null() {
                unsafe { *(**t.next()).last() = *t.last() };
            }
        }
    }
}

