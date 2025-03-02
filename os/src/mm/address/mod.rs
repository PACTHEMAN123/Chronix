//! Implementation of physical and virtual address and page number.

use core::{iter::Step, ops::{Add, AddAssign, Sub, SubAssign}};

use super::PageTableEntry;
use crate::config::{KERNEL_ADDR_OFFSET, PAGE_SIZE, PAGE_SIZE_BITS};

/// physical address
const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

mod kern;
mod phys;
mod virt;

pub use kern::{KernAddr, KernPageNum};
pub use phys::{PhysAddr, PhysPageNum};
pub use virt::{VirtAddr, VirtPageNum};
