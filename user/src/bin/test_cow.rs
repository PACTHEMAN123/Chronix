#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{brk, fork, wait, yield_};

#[no_mangle]
pub fn main() -> i32 {
    let p = brk(0);
    brk(p as usize + 1024);
    
    let x = unsafe {
        &mut *(p as *mut usize)
    };
    *x = 132;
    if fork() == 0 {
        println!("child: x = {}", x);
        *x = 169;
        println!("child: x = {}", x);
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                yield_();
                continue;
            } else {
                break;
            }
        }
        println!("parent: x = {}", x);
    }
    0
}
