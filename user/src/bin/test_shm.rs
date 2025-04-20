#![no_std]
#![no_main]
#![feature(str_from_raw_parts)]

use core::ptr::slice_from_raw_parts_mut;

use user_lib::{fork, mmap, wait, MmapFlags, MmapProt};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main(_args: &[&str]) -> i32 {
    let ptr = mmap(
        0, 1024, 
        MmapProt::PROT_READ | MmapProt::PROT_WRITE, 
        MmapFlags::MAP_ANONYMOUS | MmapFlags::MAP_SHARED, 
        0, 0
    );
    println!("{:#x}", ptr);
    let ptr = ptr as *mut u64;
    let slice = unsafe {
        &mut *slice_from_raw_parts_mut(ptr, 1024 / 8)
    };
    if fork() == 0 {
        slice[0] = 1;
        slice[1] = 1;
        for i in 2..slice.len() {
            slice[i] = (slice[i-1] + slice[i-2]) % 1000000007;
        }
        println!("child");
    } else {
        let mut exit_code = 0;
        let pid = wait(&mut exit_code);
        if pid == -1 {
            return -1;
        }
        println!("parent");
        if pid != -1 {
            for &mut i in slice {
                print!("{} ", i);
            }
            println!("");
        }
    }
    0
}
