//! quick methods to round up and down
#![allow(missing_docs)]

pub const PAGE_MASK: usize = 0xFFF;

pub fn round_down_to_page(offset: usize) -> usize {
    offset & !PAGE_MASK
}

pub fn round_up_to_page(offset: usize) -> usize {
    round_down_to_page(offset + PAGE_MASK)
}