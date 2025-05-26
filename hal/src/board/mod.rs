#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;

pub fn get_device_tree_addr() -> usize {
    unsafe extern "C" {
        fn _dtb_start();
    }
    _dtb_start as *const usize as usize
}
