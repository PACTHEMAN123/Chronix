use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::{self, Vec}};
use fatfs::info;
use hal::{addr::RangePPNHal, allocator::FrameAllocatorTrackerExt, util::smart_point::StrongArc};

use crate::{config::{BLOCK_SIZE, PAGE_SIZE}, devices::{buffer_cache, BlockDevice}, mm::{allocator::FrameAllocator, FrameTracker}, sync::mutex::SpinNoIrqLock};

// we use frame allocator to acquire block cache space
// |------------ Block id -----------|
// |- Page index -|- in page offset -| 
// |--- 64 : 3 ---|----- 2 : 0 ------|

pub struct BufferCache {
    // a mapping from block id the pointer of the buffered block
    buffer_cache: SpinNoIrqLock<BTreeMap<usize, Arc<BlockPage>>>
}

impl BufferCache {
    pub fn new() -> Self {
        Self {
            buffer_cache: SpinNoIrqLock::new(BTreeMap::new())
        }
    }
    /// get the raw buffer cache
    pub fn get_block_pages(&self) -> &SpinNoIrqLock<BTreeMap<usize, Arc<BlockPage>>> {
        &self.buffer_cache
    }
    pub fn get_block_page(&self, page_index: usize) -> Option<Arc<BlockPage>> {
        self.buffer_cache.lock().get(&page_index).cloned()
    }
    pub fn insert_block_page(&self, page_index: usize, page: Arc<BlockPage>) {
        self.buffer_cache.lock().insert(page_index, page);
    }
}

pub struct BlockPage {
    /// a page contains 4 blocks
    pub frame: StrongArc<FrameTracker>,
    pub page_index: usize,
    /// maybe we can support pre-read
    pub is_loaded: SpinNoIrqLock<[bool; 8]>
}

impl BlockPage {
    pub fn new(page_index: usize) -> Arc<Self> {
        let frame = FrameAllocator.alloc_tracker(1)
            .expect("[BlockPage]: allocating page failed");
        frame.range_ppn.get_slice_mut::<u8>().fill(0);
        Arc::new(Self {
            page_index,
            frame: StrongArc::new(frame),
            is_loaded: SpinNoIrqLock::new([false; 8]),
        })
    }

    pub fn frame(&self) -> StrongArc<FrameTracker> {
        self.frame.clone()
    }

    pub fn write_block_one(&self, in_page_offset: usize, buf: &[u8]) {
        assert_eq!(buf.len() % BLOCK_SIZE, 0);
        let slice = self.frame.range_ppn.get_slice_mut::<u8>();
        let start = BLOCK_SIZE * in_page_offset;
        let end = BLOCK_SIZE * (in_page_offset + 1);
        slice[start..end].copy_from_slice(&buf[..BLOCK_SIZE]);
    }

    pub fn read_block_one(&self, in_page_offset: usize, buf: &mut [u8]) {
        assert_eq!(buf.len() % BLOCK_SIZE, 0);
        let slice = self.frame.range_ppn.get_slice_mut::<u8>();
        let start = BLOCK_SIZE * in_page_offset;
        let end = BLOCK_SIZE * (in_page_offset + 1);
        buf[..BLOCK_SIZE].copy_from_slice(&slice[start..end]);
    }
}


impl dyn BlockDevice {

    pub fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        if let Some(buffer_cache) = self.buffer_cache() {
            let page_index = block_id / (PAGE_SIZE / BLOCK_SIZE);
            let in_page_offset = block_id % (PAGE_SIZE / BLOCK_SIZE);
            // log::info!("block_id {block_id}, page index {page_index}, in_page_offset {in_page_offset}");
            let start = BLOCK_SIZE * in_page_offset;
            let end = BLOCK_SIZE * (in_page_offset + 1);
            if let Some(page) = buffer_cache.get_block_page(page_index) {
                // page exist
                let is_loaded = page.is_loaded.lock()[in_page_offset];
                if !is_loaded {
                    // miss
                    // copy from raw device to physical frame
                    let frame = page.frame();
                    let slice = &mut frame.range_ppn.get_slice_mut::<u8>()[start..end];
                    self.direct_read_block(block_id, slice);
                    // update info
                    page.is_loaded.lock()[in_page_offset] = true;
                }
                page.read_block_one(in_page_offset, buf);
            } else {
                // page not exist
                // same as page exist but block miss
                let page = BlockPage::new(page_index);
                let frame = page.frame();
                let slice = &mut frame.range_ppn.get_slice_mut::<u8>()[start..end];
                self.direct_read_block(block_id, slice);
                page.is_loaded.lock()[in_page_offset] = true;
                page.read_block_one(in_page_offset, buf);
                buffer_cache.insert_block_page(page_index, page);
            }
        } else {
            self.direct_read_block(block_id, buf);
        }
    }

    pub fn write_block(&self, block_id: usize, buf: &[u8]) {
        if let Some(buffer_cache) = self.buffer_cache() {
            let page_index = block_id / (PAGE_SIZE / BLOCK_SIZE);
            let in_page_offset = block_id % (PAGE_SIZE / BLOCK_SIZE);
            let start = BLOCK_SIZE * in_page_offset;
            let end = BLOCK_SIZE * (in_page_offset + 1);
            if let Some(page) = buffer_cache.get_block_page(page_index) {
                // page exist
                let is_loaded = page.is_loaded.lock()[in_page_offset];
                if !is_loaded {
                    // miss
                    // copy from raw device to physical frame
                    let frame = page.frame();
                    let slice = &mut frame.range_ppn.get_slice_mut::<u8>()[start..end];
                    self.direct_read_block(block_id, slice);
                    // update info
                    page.is_loaded.lock()[in_page_offset] = true;
                }
                page.write_block_one(in_page_offset, buf);
            } else {
                // page not exist
                // same as page exist but block miss
                let page = BlockPage::new(page_index);
                let frame = page.frame();
                let slice = &mut frame.range_ppn.get_slice_mut::<u8>()[start..end];
                self.direct_write_block(block_id, slice);
                page.is_loaded.lock()[in_page_offset] = true;
                page.write_block_one(in_page_offset, buf);
                buffer_cache.insert_block_page(page_index, page);
            }
        } else {
            self.direct_write_block(block_id, buf);
        }
    }
}