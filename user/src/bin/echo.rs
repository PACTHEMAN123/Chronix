#![no_std]
#![no_main]
#![feature(str_from_raw_parts)]

use core::{ffi::c_schar, str};

#[macro_use]
extern crate user_lib;


#[no_mangle]
pub fn main(args: &[&str]) -> i32 {
    for s in &args[1..] {
        print!("{} ", s);
    }
    print!("\n");
    0
}
