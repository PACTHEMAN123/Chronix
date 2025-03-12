use core::ptr::{null_mut, slice_from_raw_parts_mut, NonNull};

use alloc::{alloc::{AllocError, Allocator}, collections::btree_map::BTreeMap};
use log::info;

use crate::{config::PAGE_SIZE, sync::{mutex::{spin_mutex::SpinMutex, Spin}, UPSafeCell}};

use super::{frame_alloc, frame_dealloc, FrameTracker};

use lazy_static::lazy_static;

/// slab allocator
pub static SLAB_ALLOCATOR_INNER: SlabAllocatorInner = SlabAllocatorInner::new();

/// Slab Allocator
#[derive(Clone)]
pub struct SlabAllocator;

/// Slab Allocator's Inner
pub struct SlabAllocatorInner {
    pub cache08: SpinMutex<SlabCache<8>, Spin>, 
    pub cache16: SpinMutex<SlabCache<16>, Spin>, 
    pub cache32: SpinMutex<SlabCache<32>, Spin>, 
    pub cache24: SpinMutex<SlabCache<24>, Spin>, 
    pub cache40: SpinMutex<SlabCache<40>, Spin>, 
    pub cache48: SpinMutex<SlabCache<48>, Spin>, 
    pub cache56: SpinMutex<SlabCache<56>, Spin>, 
    pub cache64: SpinMutex<SlabCache<64>, Spin>, 
    pub cache72: SpinMutex<SlabCache<72>, Spin>,
    pub cache80: SpinMutex<SlabCache<80>, Spin>,
    pub cache88: SpinMutex<SlabCache<88>, Spin>,
    pub cache96: SpinMutex<SlabCache<96>, Spin>, 
    pub cache192: SpinMutex<SlabCache<192>, Spin>, 
}

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

impl SlabAllocatorInner {
    /// new
    pub const fn new() -> Self {
        Self {
            cache08: SpinMutex::new(SlabCache::<8>::new()),
            cache16: SpinMutex::new(SlabCache::<16>::new()),
            cache24: SpinMutex::new(SlabCache::<24>::new()),
            cache32: SpinMutex::new(SlabCache::<32>::new()),
            cache40: SpinMutex::new(SlabCache::<40>::new()),
            cache48: SpinMutex::new(SlabCache::<48>::new()),
            cache56: SpinMutex::new(SlabCache::<56>::new()),
            cache64: SpinMutex::new(SlabCache::<64>::new()),
            cache72: SpinMutex::new(SlabCache::<72>::new()),
            cache80: SpinMutex::new(SlabCache::<80>::new()),
            cache88: SpinMutex::new(SlabCache::<88>::new()),
            cache96: SpinMutex::new(SlabCache::<96>::new()),
            cache192: SpinMutex::new(SlabCache::<192>::new()),
        }
    }

    /// release useless frame
    pub fn shrink(&self) {
        self.cache08.lock().shrink();
        self.cache16.lock().shrink();
        self.cache24.lock().shrink();
        self.cache32.lock().shrink();
        self.cache40.lock().shrink();
        self.cache48.lock().shrink();
        self.cache56.lock().shrink();
        self.cache64.lock().shrink();
        self.cache72.lock().shrink();
        self.cache80.lock().shrink();
        self.cache88.lock().shrink();
        self.cache96.lock().shrink();
        self.cache192.lock().shrink();
    }

    pub fn alloc_by_layout(&self, layout: core::alloc::Layout) -> Option<NonNull<u8>> {
        match layout.pad_to_align().size() {
            0..=8 => {
                self.cache08.lock().alloc()
            },
            9..=16 => {
                self.cache16.lock().alloc()
            },
            17..=24 => {
                self.cache24.lock().alloc()
            },
            25..=32 => {
                self.cache32.lock().alloc()
            },
            33..=40 => {
                self.cache40.lock().alloc()
            },
            41..=48 => {
                self.cache48.lock().alloc()
            },
            49..=56 => {
                self.cache56.lock().alloc()
            },
            57..=64 => {
                self.cache64.lock().alloc()
            },
            65..=72 => {
                self.cache72.lock().alloc()
            },
            73..=80 => {
                self.cache80.lock().alloc()
            },
            81..=88 => {
                self.cache88.lock().alloc()
            },
            89..=96 => {
                self.cache96.lock().alloc()
            },
            97..=192 => {
                self.cache192.lock().alloc()
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
                self.cache08.lock().dealloc(ptr);
            },
            9..=16 => {
                self.cache16.lock().dealloc(ptr);
            },
            17..=24 => {
                self.cache24.lock().dealloc(ptr);
            },
            25..=32 => {
                self.cache32.lock().dealloc(ptr);
            },
            33..=40 => {
                self.cache40.lock().dealloc(ptr);
            },
            41..=48 => {
                self.cache48.lock().dealloc(ptr);
            },
            49..=56 => {
                self.cache56.lock().dealloc(ptr);
            },
            57..=64 => {
                self.cache64.lock().dealloc(ptr);
            },
            65..=72 => {
                self.cache72.lock().dealloc(ptr);
            },
            73..=80 => {
                self.cache80.lock().dealloc(ptr);
            },
            81..=88 => {
                self.cache88.lock().dealloc(ptr);
            },
            89..=96 => {
                self.cache96.lock().dealloc(ptr);
            },
            97..=192 => {
                self.cache192.lock().dealloc(ptr);
            },
            _ => {}
        }
    }

    /// dealloc a payload
    pub fn dealloc<T: Sized>(&self, ptr: NonNull<T>) {
        self.dealloc_by_layout(ptr.cast(), core::alloc::Layout::new::<T>());
    }
}

/// alloc from slab allocator
pub fn slab_alloc<T: Sized>() -> Option<NonNull<T>> {
    SLAB_ALLOCATOR_INNER.alloc() 
}

/// dealloc to slab allocator
pub fn slab_dealloc<T: Sized>(ptr: NonNull<T>) {
    SLAB_ALLOCATOR_INNER.dealloc(ptr); 
}

/// shrink the slab
#[allow(unused)]
pub fn slab_shrink() {
    unsafe { SLAB_ALLOCATOR_INNER.shrink(); }
}

#[repr(C)]
#[allow(missing_docs)]
struct SlabBlock {
    next: *mut SlabBlock,
    belong: KernAddr,
    size: usize
}

#[repr(C)]
#[allow(missing_docs)]
pub union FreeNode<const S: usize> {
    next: *mut FreeNode<S>,
    data: [u8; S]
}

#[allow(missing_docs)]
pub struct SlabCache<const S: usize> {
    head: *mut SlabBlock,
    freelist: *mut FreeNode<S>
}

unsafe impl<const S: usize> Send for SlabCache<S> {

}

#[allow(unused)]
impl<const S: usize> SlabCache<S> {

    /// 初始化
    pub const fn new() -> Self {
        // // S 不能太大
        // assert_ne!(Self::block_cap(), 0);
        Self {
            head: null_mut(),
            freelist: null_mut()
        }
    }

    /// 每页容量
    pub const fn block_cap() -> usize {
        (PAGE_SIZE - size_of::<SlabBlock>()) / S
    }

    /// 分配一个载荷
    pub fn alloc<T: Sized>(&mut self) -> Option<NonNull<T>> {
        assert!(size_of::<T>() <= S);
        loop {
            if self.freelist.is_null() { // 空闲链表为空，需要申请新的页
                info!("[SlabCache] new frame");
                let new_ppn = frame_alloc()?.leak(); // 不需要RAII，leak获得页号
                let block = new_ppn.to_kern().get_mut::<SlabBlock>(); // 页面元信息
                block.next = self.head;
                self.head = block; // 将新页加入页链表
                block.belong = KernAddr(self as *mut SlabCache<S> as usize);
                block.size = 0; // 因为是新页，size置零
                let node_start_pa = PhysAddr::from(new_ppn) + size_of::<SlabBlock>(); // 数据节点列表开头的物理地址
                let nodes = unsafe {
                    &mut *slice_from_raw_parts_mut(node_start_pa.to_kern().get_mut::<FreeNode<S>>(), Self::block_cap())
                };
                for i in 0..nodes.len()-1 {
                    nodes[i].next = &mut nodes[i+1]
                }
                nodes[nodes.len()-1].next = null_mut(); // 建立链表
                self.freelist = &mut nodes[0]; // 加入空闲链表
            } else {
                let payload = self.freelist;
                self.freelist = unsafe { (*self.freelist).next };
                let payload_ka = KernAddr(payload as usize); // 载荷的内核地址
                let block = payload_ka.floor().get_mut::<SlabBlock>(); // 页面元信息
                block.size += 1; // 已分配大小+1
                unsafe { 
                    let payload = &mut *slice_from_raw_parts_mut(
                        payload as *mut u8, 
                        size_of::<FreeNode::<S>>()
                    );
                    payload.fill(0);
                } // 清空
                return Some(NonNull::new(payload as *mut T).unwrap());
            }
        }  
    }

    /// 回收载荷
    pub fn dealloc<T: Sized>(&mut self, payload: NonNull<T>) {
        let payload_ka = KernAddr(payload.as_ptr() as usize);
        let block = payload_ka.floor().get_mut::<SlabBlock>();
        if block.belong.0 != self as *mut SlabCache<S> as usize {
            panic!("[SlabCache] dealloc a payload to a wrong cache, expect: {:#x}, actually {:#x}", 
                block.belong.0, 
                self as *mut SlabCache<S> as usize
            );
        }
        let node = payload_ka.get_mut::<FreeNode<S>>();
        node.next = self.freelist;
        self.freelist = node;
        block.size -= 1;
    }

    /// 释放无用页
    pub fn shrink(&mut self) {
        if self.head.is_null() || self.freelist.is_null() {
            return;
        }

        // 先清理freelist
        let mut last = self.freelist;
        let mut cur = unsafe { (*self.freelist).next };
        while !cur.is_null() {
            let block = KernAddr(cur as usize).floor().get_mut::<SlabBlock>();
            if block.size == 0 {
                unsafe { (*last).next = (*cur).next };
                cur = unsafe { (*cur).next };
            } else {
                last = cur;
                cur = unsafe { (*cur).next };
            }
        }

        {
            let block = KernAddr(self.freelist as usize).floor().get_mut::<SlabBlock>();
            if block.size == 0 {
                self.freelist = unsafe { (*self.freelist).next };
            }
        }

        let mut pre_ref = unsafe { &mut *self.head }; // 先跳过头节点
        let mut cur = pre_ref.next;
        while !cur.is_null() {
            let cur_ref = unsafe { &mut *cur };
            if cur_ref.size == 0 { // 若页没有使用
                let ppn = KernAddr(cur as usize).to_phys().floor(); // 不能马上dealloc，因为后面还要读cur->next
                pre_ref.next = cur_ref.next; // 修改pre->next，指向cur->next
                cur = cur_ref.next; // cur 向后移动
                frame_dealloc(ppn);
            } else {
                pre_ref = unsafe { &mut *pre_ref.next }; // pre 向后移动
                cur = cur_ref.next; // cur 向后移动
            }
        }
        // 最后处理头节点
        if unsafe { (*self.head).size } == 0 { // 若页没有使用
            let ppn = KernAddr(self.head as usize).to_phys().floor(); // 不能马上dealloc，因为后面还要读self.head->next
            self.head = unsafe { (*self.head).next };
            frame_dealloc(ppn);
        }
    }
}