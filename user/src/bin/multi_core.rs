#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, fork, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    fork();
    println!("Hello, world 1!");
    fork();
    println!("hello, world again 2!");
    fork();
    println!("hello, world again again 3 !");
    fork();
    println!("hello, world again again again 4!");
    fork();
    println!("hello, world again again again 5!");
    0
}