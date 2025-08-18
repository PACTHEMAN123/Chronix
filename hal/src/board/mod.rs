#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;

pub static mut DTB_ADDR: usize = 0;

pub(crate) fn set_device_tree_addr(addr: usize) {
    unsafe { DTB_ADDR = addr; }
}


pub fn get_device_tree_addr() -> usize {
    unsafe { DTB_ADDR as *const usize as usize }
}
