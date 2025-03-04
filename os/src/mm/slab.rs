use core::ptr::{null_mut, slice_from_raw_parts_mut, NonNull};

use alloc::collections::btree_map::BTreeMap;

use crate::{config::PAGE_SIZE, mm::{KernAddr, PhysAddr}, sync::UPSafeCell};

use super::{frame_alloc, frame_dealloc, FrameTracker, PhysPageNum};

use lazy_static::lazy_static;


lazy_static! {
    /// slab allocator
    pub static ref SLAB_ALLOCATOR: UPSafeCell<SlabAllocator> = 
        unsafe { UPSafeCell::new(SlabAllocator::new()) };
}

pub struct SlabAllocator {
    pub cache08: SlabCache<8>, 
    pub cache16: SlabCache<16>, 
    pub cache24: SlabCache<24>, 
    pub cache32: SlabCache<32>, 
    pub cache40: SlabCache<40>, 
    pub cache48: SlabCache<48>, 
    pub cache56: SlabCache<56>, 
    pub cache64: SlabCache<64>, 
    pub cache72: SlabCache<72>,
    pub cache80: SlabCache<80>,
    pub cache88: SlabCache<88>,
    pub cache96: SlabCache<96>, 
}

impl SlabAllocator {

    pub fn new() -> Self {
        Self {
            cache08: SlabCache::<8>::new(),
            cache16: SlabCache::<16>::new(),
            cache24: SlabCache::<24>::new(),
            cache32: SlabCache::<32>::new(),
            cache40: SlabCache::<40>::new(),
            cache48: SlabCache::<48>::new(),
            cache56: SlabCache::<56>::new(),
            cache64: SlabCache::<64>::new(),
            cache72: SlabCache::<72>::new(),
            cache80: SlabCache::<80>::new(),
            cache88: SlabCache::<88>::new(),
            cache96: SlabCache::<96>::new(),
        }
    }

    pub fn alloc<T: Sized>(&mut self) -> Option<NonNull<T>> {
        match size_of::<T>() {
            0..=8 => {
                Some(self.cache08.alloc())
            },
            9..=16 => {
                Some(self.cache16.alloc())
            },
            17..=24 => {
                Some(self.cache24.alloc())
            },
            25..=32 => {
                Some(self.cache32.alloc())
            },
            33..=40 => {
                Some(self.cache40.alloc())
            },
            41..=48 => {
                Some(self.cache48.alloc())
            },
            49..=56 => {
                Some(self.cache56.alloc())
            },
            57..=64 => {
                Some(self.cache64.alloc())
            },
            65..=72 => {
                Some(self.cache72.alloc())
            },
            73..=80 => {
                Some(self.cache80.alloc())
            },
            81..=88 => {
                Some(self.cache88.alloc())
            },
            89..=96 => {
                Some(self.cache96.alloc())
            },
            97.. => {
                None
            }
        }
    }

    pub fn dealloc<T: Sized>(&mut self, ptr: NonNull<T>) {
        match size_of::<T>() {
            0..=8 => {
                self.cache08.dealloc(ptr);
            },
            9..=16 => {
                self.cache16.dealloc(ptr);
            },
            17..=24 => {
                self.cache24.dealloc(ptr);
            },
            25..=32 => {
                self.cache32.dealloc(ptr);
            },
            33..=40 => {
                self.cache40.dealloc(ptr);
            },
            41..=48 => {
                self.cache48.dealloc(ptr);
            },
            49..=56 => {
                self.cache56.dealloc(ptr);
            },
            57..=64 => {
                self.cache64.dealloc(ptr);
            },
            65..=72 => {
                self.cache72.dealloc(ptr);
            },
            73..=80 => {
                self.cache80.dealloc(ptr);
            },
            81..=88 => {
                self.cache88.dealloc(ptr);
            },
            89..=96 => {
                self.cache96.dealloc(ptr);
            },
            97.. => {}
        }
    }
}

/// alloc from slab allocator
pub fn slab_alloc<T: Sized>() -> NonNull<T> {
    SLAB_ALLOCATOR.exclusive_access().alloc().unwrap()
}

/// dealloc to slab allocator
pub fn slab_dealloc<T: Sized>(ptr: NonNull<T>){
    SLAB_ALLOCATOR.exclusive_access().dealloc(ptr);
}


#[repr(C)]
#[allow(missing_docs, unused)]
struct SlabBlock {
    next: *mut SlabBlock,
    size: usize
}

#[repr(C)]
#[allow(missing_docs, unused)]
pub union FreeNode<const S: usize> {
    next: *mut FreeNode<S>,
    data: [u8; S]
}

#[allow(missing_docs, unused)]
pub struct SlabCache<const S: usize> {
    head: *mut SlabBlock,
    freelist: *mut FreeNode<S>
}

unsafe impl<const S: usize> Send for SlabCache<S> {

}

#[allow(unused)]
impl<const S: usize> SlabCache<S> {

    /// 初始化
    pub fn new() -> Self {
        // S 不能太大
        assert_ne!(Self::block_cap(), 0);
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
    pub fn alloc<T: Sized>(&mut self) -> NonNull<T> {
        assert!(size_of::<T>() <= S);
        loop {
            if self.freelist.is_null() { // 空闲链表为空，需要申请新的页
                let new_ppn = frame_alloc().unwrap().leak(); // 不需要RAII，leak获得页号
                let block = new_ppn.to_kern().get_mut::<SlabBlock>(); // 页面元信息
                block.next = self.head;
                self.head = block; // 将新页加入页链表
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
                return NonNull::new(payload as *mut T).unwrap();
            }
        }  
    }

    /// 回收载荷
    pub fn dealloc<T: Sized>(&mut self, payload: NonNull<T>) {
        let payload_ka = KernAddr(payload.as_ptr() as usize);
        let node = payload_ka.get_mut::<FreeNode<S>>();
        node.next = self.freelist;
        self.freelist = node;
        let block = payload_ka.floor().get_mut::<SlabBlock>();
        block.size -= 1;
    }

    /// 释放无用页
    pub fn shrink(&mut self) {
        if self.head.is_null() {
            return;
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