#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{chdir, execve, fork, waitpid};

fn run_cmd(cmd: &str) {
    let pid = fork();
    if pid == 0 {
        // default use musl busybox
        execve(
            "/musl/busybox",
            &["busybox", "sh", "-c", cmd],
            &[
                "PATH=/:/bin",
                "HOME=/home/chronix",
            ],
        );
    } else if pid > 0 {
        let mut result: i32 = 0;
        waitpid(pid as usize, &mut result);
    }
}

fn run_test() {
    run_cmd("./basic_testcode.sh");
    run_cmd("./busybox_testcode.sh");
    run_cmd("./lua_testcode.sh");
    run_cmd("./libcbench_testcode.sh");
    run_cmd("./libctest_testcode.sh");
    run_cmd("./lmbench_testcode.sh");
    run_cmd("./iozone_testcode.sh");
    run_cmd("./cyclictest_testcode.sh");
    // TODOS
    // run_cmd("./iperf_testcode.sh")
    // run_cmd("./netperf") ?
    // run_cmd("./ltp_testcode.sh")
}

#[no_mangle]
fn main() -> i32 {
    run_cmd("/musl/busybox --install /bin");
    run_cmd("rm /bin/sh");

    println!("start to run musl test");
    chdir("/musl\0");
    run_test();
    println!("finish running musl test");

    println!("start to run glibc test");
    chdir("/glibc\0");
    run_test();
    println!("finish running glibc test");

    0
}
