#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::fork;

#[no_mangle]
pub fn main() -> i32 {
    let mut res = 1.5 +3.4;
    if fork() == 0 {
        res += 7.6;
        println!("res in child = {}", res);
    } else {
        println!("res in parent = {}", res);
    };
    0
}