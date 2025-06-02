#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{execve, fork, getpid, kill, shutdown, sigaction, sleep, wait, yield_, SignalAction, SignalFlags, SIGKILL, SIGTERM};

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

fn term_sig_handler(_signo: i32) {
    println!("[initproc] term_sig_handler");
    kill(-1, SIGTERM);
    sleep(500);
    kill(-1, SIGKILL);
    shutdown();
    loop { yield_(); }
}


#[no_mangle]
fn main() -> i32 {
    init_env();
    let initproc_pid = getpid();
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
        println!("[secondproc] execve busybox fail");
        kill(initproc_pid, SIGTERM);
    } else {
        let term_sig_action = SignalAction { handler: term_sig_handler as *const fn(i32) as usize, mask: SignalFlags::all() };
        sigaction(SIGTERM, Some(&term_sig_action), None);
        println!("into user mode initproc wait");
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            // -10: ECHILD
            if pid == -10 {
                println!("[initproc] no child process, shutdown");
                kill(initproc_pid, SIGTERM);
            }
            println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid, exit_code,
            );
        }
    }
    0
}
