#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, execve, fork, wait, yield_};

fn run_cmd(cmd: &str) {
    if fork() == 0 {
        // default use musl busybox
        #[cfg(target_arch="riscv64")]
        execve(
            "/riscv/musl/busybox",
            &["busybox", "sh", "-c", cmd],
            &[
                "PATH=/:/bin",
                "HOME=/home/chronix",
            ],
        );

        #[cfg(target_arch="loongarch64")]
        execve(
            "/loongarch/musl/busybox",
            &["busybox", "sh", "-c", cmd],
            &[
                "PATH=/:/bin",
                "HOME=/home/chronix",
            ],
        );
    } else {
        let mut result: i32 = 0;
        wait(&mut result);
    }
}

fn init_env() {
    #[cfg(target_arch="riscv64")]
    run_cmd("/riscv/musl/busybox --install /bin");
    #[cfg(target_arch="loongarch64")]
    run_cmd("/loongarch/musl/busybox --install /bin");
    run_cmd("rm /bin/sh");
}

#[no_mangle]
fn main() -> i32 {
    init_env();

    println!("into user mode initproc");
    if fork() == 0 {
        println!("into user mode initproc fork");
        #[cfg(target_arch="riscv64")]
        execve(
            "/riscv/musl/busybox",
            &["busybox", "sh"],
            &[
                "PATH=/:/bin",
                "TERM=screen",
            ],
        );
        #[cfg(target_arch="loongarch64")]
        execve(
            "/loongarch/musl/busybox",
            &["busybox", "sh"],
            &[
                "PATH=/:/bin",
                "TERM=screen",
            ],
        );
    } else {
        println!("into user mode initproc wait");
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                // println!("in pid == -1");
                return -1;
            }
            println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid, exit_code,
            );
        }
    }
    0
}
