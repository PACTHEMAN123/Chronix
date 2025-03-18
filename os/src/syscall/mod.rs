//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

const SYSCALL_GETCWD: usize = 17;
const SYSCALL_DUP: usize = 23;
const SYSCALL_DUP3: usize = 24;
const SYSCALL_MKDIR: usize = 34;
const SYSCALL_CHDIR: usize = 49;
const SYSCALL_OPENAT: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_RT_SIGACTION: usize = 134;
const SYSCALL_RT_SIGPROCMASK: usize = 135;
const SYSCALL_RT_SIGRETURN: usize = 139;
const SYSCALL_GETTIMEOFDAY: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_CLONE: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_BRK: usize = 214;

pub mod fs;
pub mod process;
pub mod time;
pub mod signal;

pub use fs::*;
pub use process::*;
pub use time::*;
pub use signal::*;

use crate::{signal::{SigAction, SigSet}, timer::ffi::TimeVal};

/// handle syscall exception with `syscall_id` and other arguments
pub async fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id { 
        SYSCALL_GETCWD => sys_getcwd(args[0] as usize, args[1] as usize),
        SYSCALL_DUP => sys_dup(args[0] as usize),
        SYSCALL_DUP3 => sys_dup3(args[0] as usize, args[1] as usize, args[2] as u32),
        SYSCALL_OPENAT => sys_openat(args[0] as isize , args[1] as *const u8, args[2] as u32, args[3] as u32),
        SYSCALL_MKDIR => sys_mkdirat(args[0] as isize, args[1] as *const u8, args[2] as usize),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe2(args[0] as *mut i32, args[1] as u32),
        SYSCALL_READ => sys_read(args[0], args[1] , args[2]).await,
        SYSCALL_WRITE => sys_write(args[0], args[1] , args[2]).await,
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield().await,
        SYSCALL_KILL => sys_kill(args[0] as isize, args[1] as i32),
        SYSCALL_RT_SIGACTION => sys_rt_sigaction(args[0] as i32, args[1] as *const SigAction, args[2] as *mut SigAction),
        SYSCALL_RT_SIGPROCMASK => sys_rt_sigprocmask(args[0] as i32, args[1] as *const u32, args[2] as *mut SigSet),
        SYSCALL_RT_SIGRETURN => sys_rt_sigreturn(),
        SYSCALL_GETTIMEOFDAY => sys_gettimeofday(args[0] as *mut TimeVal),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_CLONE => sys_clone(args[0], args[1].into(), args[2].into(), args[3].into(), args[4].into()),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1], args[2] as i32).await,
        SYSCALL_EXEC => sys_exec(args[0] , args[1] ).await,
        SYSCALL_BRK => sys_brk(hal::addr::VirtAddr(args[0])),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
