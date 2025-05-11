#![no_std]
#![no_main]
#![feature(str_from_raw_parts)]

use core::ptr::slice_from_raw_parts_mut;

use user_lib::{fork, mmap, mremap, wait, MmapFlags, MmapProt, MremapFlags};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main(_args: &[&str]) -> i32 {
    let ptr = mmap(
        0, 1024, 
        MmapProt::PROT_READ | MmapProt::PROT_WRITE, 
        MmapFlags::MAP_ANONYMOUS | MmapFlags::MAP_PRIVATE, 
        0, 0
    ) as *mut u64;
    println!("{:#x}", ptr as usize);
    let slice = unsafe {
        &mut *slice_from_raw_parts_mut(ptr, 1024 / 8)
    };
    slice[0] = 1;
    slice[1] = 1;
    for i in 2..slice.len() {
        slice[i] = (slice[i-1] + slice[i-2]) % 1000000007;
    }

    let ret = mremap(
        ptr as usize, 1024,
        8192, MremapFlags::MAYMOVE | MremapFlags::DONTUNMAP,
        0
    );
    if ret < 0 {
        println!("error: {}", ret);
        return ret as i32;
    }
    let ptr2 = ret as *mut u64;
    println!("{:#x}", ptr2 as usize);
    let slice2 = unsafe {
        &mut *slice_from_raw_parts_mut(ptr2, 1024 / 8)
    };
    
    for &mut i in slice {
        print!("{} ", i);
    }
    println!("");
    for &mut i in slice2 {
        print!("{} ", i);
    }
    println!("");
    
    0
}
