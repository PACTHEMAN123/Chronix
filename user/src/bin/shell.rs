#![no_std]
#![no_main]

extern crate user_lib;

extern crate alloc;

use user_lib::{execve, fork, println, wait, waitpid};

fn run_cmd(cmd: &str) {
    if fork() == 0 {
        execve(
            "busybox",
            &["busybox", "sh", "-c", cmd],
            &[
                "PATH=/:/bin",
                "HOME=/home/chronix",
                //"LD_LIBRARY_PATH=/:/lib:/lib/glibc/:/lib/musl",
            ],
        );
    } else {
        let mut result: i32 = 0;
        waitpid((-1isize) as usize, &mut result);
    }
}

#[no_mangle]
fn main() -> i32 {
    println!("start to init busybox");
    run_cmd("busybox --install /bin");
    run_cmd("rm /bin/sh");
    println!("install finished, start to run busy box");
    execve(
        "busybox",
        &["busybox", "sh"],
        &[
            "PATH=/:/bin",
            //"LD_LIBRARY_PATH=/:/lib:/lib/glibc/:/lib/musl",
            "TERM=screen",
        ],
    );
    0
}