//! signal module for kernel
#![allow(missing_docs)]

mod action;
mod handler;
mod manager;

pub use action::*;
pub use handler::*;
pub use manager::*;

use crate::task::current_task;

/// Hangup detected on controlling terminal
/// or death of controlling process
pub const SIGHUP: usize = 1;
/// Interrupt from keyboard
pub const SIGINT: usize = 2;
/// Quit from keyboard
pub const SIGQUIT: usize = 3;
/// Illegal Instruction
pub const SIGILL: usize = 4;
/// Trace/breakpoint trap
pub const SIGTRAP: usize = 5;
/// Abort signal from abort(3)
pub const SIGABRT: usize = 6;
/// Bus error (bad memory access)
pub const SIGBUS: usize = 7;
/// Erroneous arithmetic operation
pub const SIGFPE: usize = 8;
/// Kill signal
pub const SIGKILL: usize = 9;
/// User-defined signal 1
pub const SIGUSR1: usize = 10;
/// Invalid memory reference
pub const SIGSEGV: usize = 11;
/// User-defined signal 2
pub const SIGUSR2: usize = 12;
/// Broken pipe: write to pipe with no readers; see pipe(7)
pub const SIGPIPE: usize = 13;
/// Timer signal from alarm(2)
pub const SIGALRM: usize = 14;
/// Termination signal
pub const SIGTERM: usize = 15;
/// Stack fault on coprocessor (unused)
pub const SIGSTKFLT: usize = 16;
/// Child stopped or terminated
pub const SIGCHLD: usize = 17;
/// Continue if stopped
pub const SIGCONT: usize = 18;
/// Stop process
pub const SIGSTOP: usize = 19;
/// Stop typed at terminal
pub const SIGTSTP: usize = 20;
/// Terminal input for background process
pub const SIGTTIN: usize = 21;
/// Terminal output for background process
pub const SIGTTOU: usize = 22;
/// Urgent condition on socket (4.2BSD)
pub const SIGURG: usize = 23;
/// CPU time limit exceeded (4.2BSD);
/// see setrlimit(2)
pub const SIGXCPU: usize = 24;
/// File size limit exceeded (4.2BSD);
/// see setrlimit(2)
pub const SIGXFSZ: usize = 25;
/// Virtual alarm clock (4.2BSD)
pub const SIGVTALRM: usize = 26;
/// Profiling timer expired
pub const SIGPROF: usize = 27;
/// Window resize signal (4.3BSD, Sun)
pub const SIGWINCH: usize = 28;
/// I/O now possible (4.2BSD)
pub const SIGIO: usize = 29;
/// Power failure (System V)
pub const SIGPWR: usize = 30;
/// Bad system call (SVr4);
pub const SIGSYS: usize = 31;
pub const SIGRTMIN: usize = 32;
pub const SIGRT_1: usize = SIGRTMIN + 1;
pub const SIG_NUM: usize = 33;


bitflags! {
    pub struct SigSet: usize {
        const SIGHUP    = 1 << (SIGHUP -1);
        const SIGINT    = 1 << (SIGINT - 1);
        const SIGQUIT   = 1 << (SIGQUIT - 1);
        const SIGILL    = 1 << (SIGILL - 1);
        const SIGTRAP   = 1 << (SIGTRAP - 1);
        const SIGABRT   = 1 << (SIGABRT - 1);
        const SIGBUS    = 1 << (SIGBUS - 1);
        const SIGFPE    = 1 << (SIGFPE - 1);
        const SIGKILL   = 1 << (SIGKILL - 1);
        const SIGUSR1   = 1 << (SIGUSR1 - 1);
        const SIGSEGV   = 1 << (SIGSEGV - 1);
        const SIGUSR2   = 1 << (SIGUSR2 - 1);
        const SIGPIPE   = 1 << (SIGPIPE - 1);
        const SIGALRM   = 1 << (SIGALRM - 1);
        const SIGTERM   = 1 << (SIGTERM - 1);
        const SIGSTKFLT = 1 << (SIGSTKFLT- 1);
        const SIGCHLD   = 1 << (SIGCHLD - 1);
        const SIGCONT   = 1 << (SIGCONT - 1);
        const SIGSTOP   = 1 << (SIGSTOP - 1);
        const SIGTSTP   = 1 << (SIGTSTP - 1);
        const SIGTTIN   = 1 << (SIGTTIN - 1);
        const SIGTTOU   = 1 << (SIGTTOU - 1);
        const SIGURG    = 1 << (SIGURG - 1);
        const SIGXCPU   = 1 << (SIGXCPU - 1);
        const SIGXFSZ   = 1 << (SIGXFSZ - 1);
        const SIGVTALRM = 1 << (SIGVTALRM - 1);
        const SIGPROF   = 1 << (SIGPROF - 1);
        const SIGWINCH  = 1 << (SIGWINCH - 1);
        const SIGIO     = 1 << (SIGIO - 1);
        const SIGPWR    = 1 << (SIGPWR - 1);
        const SIGSYS    = 1 << (SIGSYS - 1);
        const SIGRTMIN  = 1 << (SIGRTMIN- 1);
        const SIGRT_1   = 1 << (SIGRT_1 - 1);
    }
}

impl SigSet {
    pub fn add_sig(&mut self, signo: usize) {
        self.insert(SigSet::from_bits(1 << (signo - 1)).unwrap());
    }

    pub fn contain_sig(&self, signo: usize) -> bool {
        self.contains(SigSet::from_bits(1 << (signo - 1)).unwrap())
    }

    pub fn remove_sig(&mut self, signo: usize) {
        self.remove(SigSet::from_bits(1 << (signo - 1)).unwrap())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// Unix signal info structure
pub struct SigInfo {
    /// sig id
    pub si_signo: usize,
    /// si_code for coming source
    pub si_code: i32,
    /// pid of sender
    pub si_pid: Option<usize>,
}

impl SigInfo {
    /// sent by kill, sigsend, raise
    pub const USER: i32 = 0;
    /// sent by the kernel from somewhere
    pub const KERNEL: i32 = 0x80;
    /// sent by sigqueue
    pub const QUEUE: i32 = -1;
    /// sent by timer expiration
    pub const TIMER: i32 = -2;
    /// sent by real time mesq state change
    pub const MESGQ: i32 = -3;
    /// sent by AIO completion
    pub const ASYNCIO: i32 = -4;
    /// sent by queued SIGIO
    pub const SIGIO: i32 = -5;
    /// sent by tkill system call
    pub const TKILL: i32 = -6;
    /// sent by execve() killing subsidiary threads
    pub const DETHREAD: i32 = -7;
    /// sent by glibc async name lookup completion
    pub const ASYNCNL: i32 = -60;

    // SIGCHLD si_codes
    /// child has exited
    pub const CLD_EXITED: i32 = 1;
    /// child was killed
    pub const CLD_KILLED: i32 = 2;
    /// child terminated abnormally
    pub const CLD_DUMPED: i32 = 3;
    /// traced child has trapped
    pub const CLD_TRAPPED: i32 = 4;
    /// child has stopped
    pub const CLD_STOPPED: i32 = 5;
    /// stopped child has continued
    pub const CLD_CONTINUED: i32 = 6;
    pub const NSIGCHLD: i32 = 6;
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct LinuxSigInfo {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
    pub _pad: [i32; 29],
    _align: [u64; 0],
}