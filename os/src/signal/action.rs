//! signal action

use hal::println;

use super::*;


#[derive(Debug, Clone, Copy)]
#[repr(C)]
/// signal action struct under riscv-linux arch
pub struct SigAction {
    /// sa_handler specifies the action to be associated with signum
    pub sa_handler: usize,
    /// sa_flags specifies a set of flags which modify the behavior of the signal.
    pub sa_flags: u32,
    /// The sa_restorer field is not intended for application use.
    pub sa_restorer: usize,
    /// sa_mask specifies a mask of signals which should be blocked
    pub sa_mask: [SigSet; 1],
}


bitflags! {
    /// sa_flags:
    pub struct SigActionFlag : u32 {
        /// If signum is SIGCHLD, do not receive notification when
        /// child processes stop
        const SA_NOCLDSTOP = 1;
        /// If signum is SIGCHLD, do not transform children into
        /// zombies when they terminate. 
        const SA_NOCLDWAIT = 2;
        /// The signal handler takes three arguments, not one.
        const SA_SIGINFO = 4;
        /// Call the signal handler on an alternate signal stack
        /// provided by sigaltstack(2).
        const SA_ONSTACK = 0x08000000;
        /// Provide behavior compatible with BSD signal semantics by
        /// making certain system calls restartable across signals.
        const SA_RESTART = 0x10000000;
        /// Do not add the signal to the thread's signal mask while the
        /// handler is executing, unless the signal is specified in
        /// act.sa_mask. 
        const SA_NODEFER = 0x40000000;
        /// Restore the signal action to the default upon entry to the
        /// signal handler.
        const SA_RESETHAND = 0x80000000;
        /// Not intended for application use.  This flag is used by C
        /// libraries to indicate that the sa_restorer field contains
        /// the address of a "signal trampoline".
        const SA_RESTORER = 0x04000000;
    }
}

#[derive(Debug, Clone, Copy)]
/// signal action warpper for kernel
pub struct KSigAction {
    /// inner sigaction
    pub sa: SigAction,
    /// is user defined?
    pub is_user: bool,
}

impl KSigAction {
    pub fn new(signo: usize, is_user_defined: bool) -> Self {
        let sa_handler = get_default_handler(signo);
        Self {
            is_user: is_user_defined,
            sa: SigAction {
                sa_handler,
                sa_mask: [SigSet::from_bits(0).unwrap(); 1],
                sa_flags: 0,
                sa_restorer: 0,
            }
        }
    }
}

