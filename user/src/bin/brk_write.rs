#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::brk;

#[no_mangle]
pub fn main() -> i32 {
    let p = brk(0);
    println!("{:#x}", p);
    // brk 16GiB, to test lazy allocation
    let alloc_1 = brk(p as usize + 16 * 1024 * 1024 * 1024);
    println!("{:#x}", alloc_1);
    let alloc_2 = brk(alloc_1 as usize + 16 * 1024 * 1024 * 1024);
    println!("{:#x}", alloc_2);
    let alloc_3 = brk(alloc_1 as usize + 64);
    println!("{:#x}", alloc_3);
    let x = unsafe {
        &mut *(alloc_1 as *mut usize)
    };
    *x = 132;
    println!("x: {}", x);
    0
}
