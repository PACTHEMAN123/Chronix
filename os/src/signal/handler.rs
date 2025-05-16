//! signal handlers (aka.)
//! Signal disposition
//! Each signal has a current disposition, which determines how the
//! process behaves when it is delivered the signal.
//! five handlers for five default Action

use log::*;

use crate::{signal::{SigSet, SIGABRT, SIGALRM, SIGBUS, SIGCHLD, SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, SIGIO, SIGKILL, SIGPIPE, SIGPROF, SIGPWR, SIGQUIT, SIGRTMAX, SIGSEGV, SIGSTKFLT, SIGSTOP, SIGSYS, SIGTERM, SIGTRAP, SIGTSTP, SIGTTIN, SIGTTOU, SIGURG, SIGUSR1, SIGUSR2, SIGVTALRM, SIGWINCH, SIGXCPU, SIGXFSZ}, task::current_task};

pub const SIG_ERR: usize = usize::MAX;
/// when sig_handler is set to SIG_DFL
/// should redirect to signal's default handler
/// using get_default_handler
pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;

/// handlers for Term
/// terminate the process.
pub fn term_sig_handler(signo: usize) {
    let task = current_task().unwrap().clone();
    info!("[term_sig_handler]: task {} recv sig {}, terminated", task.pid(), signo);

    // exit all the members of a thread group (process)
    task.with_thread_group(|tg| {
        for t in tg.iter() {
            t.set_zombie();
        }
    });

    // set the exit code
    task.set_exit_code(signo as i32 & 0x7f);
}

/// handlers for Ign
/// ignore the signal
pub fn ign_sig_handler(signo: usize) {
    let task = current_task().unwrap().clone();
    info!("[ign_sig_handler]: task {} recv sig {}, do nothing", task.pid(), signo);
    // do nothing
}

/// handlers for Core
/// Default action is to terminate the process and dump core
/// The default action of certain signals is to cause a process to
/// terminate and produce a core dump file, a file containing an image
/// of the process's memory at the time of termination. 
pub fn core_sig_handler(signo: usize) {
    let task = current_task().unwrap().clone();
    info!("[core_sig_handler]: task {} recv sig {}, terminated and coredump", task.pid(), signo);
    task.with_thread_group(|tg| {
        for t in tg.iter() {
            // info!("[core_sig_handler]: set task {} to zombie", t.tid());
            t.set_zombie();
        }
    })
    // todo: produce a core dump file?
}

/// handlers for Stop
/// stop the process.
pub fn stop_sig_handler(signo: usize) {
    let task = current_task().unwrap().clone();
    info!("[stop_sig_handler]: task {} recv sig {}, stop", task.pid(), signo);

    task.with_thread_group(|tg| {
        for t in tg.iter() {
            // set the task status as stopped
            t.set_stopped();
            // the task should be wake up by SIGCONT
            t.set_wake_up_sigs(SigSet::SIGCONT);
        }
    })
}

/// handlers for Cont
/// continue the process if it is currently stopped.
pub fn cont_sig_handler(signo: usize) {
    let task = current_task().unwrap().clone();
    info!("[cont_sig_handler]: task {} recv sig {}, continue", task.pid(), signo);

    task.with_thread_group(|tg| {
        for t in tg.iter() {
            if t.is_stopped() {
                t.set_running();
                t.wake();
            }
        }
    })
}

/// get the default "Action" (here, aka. handlers) of given signo
pub fn get_default_handler(signo: usize) -> usize {
    assert!(signo <= SIGRTMAX);
    let handler = match signo {
        SIGHUP => term_sig_handler,
        SIGINT => term_sig_handler,
        SIGQUIT => core_sig_handler,
        SIGILL => core_sig_handler,
        SIGTRAP => core_sig_handler,
        SIGABRT => core_sig_handler,
        SIGBUS => core_sig_handler,
        SIGFPE => core_sig_handler,
        SIGKILL => term_sig_handler,
        SIGUSR1 => term_sig_handler,
        SIGSEGV => core_sig_handler,
        SIGUSR2 => term_sig_handler,
        SIGPIPE => term_sig_handler,
        SIGALRM => term_sig_handler,
        SIGTERM => term_sig_handler,
        SIGSTKFLT => term_sig_handler,
        SIGCHLD => ign_sig_handler,
        SIGCONT => cont_sig_handler,
        SIGSTOP => stop_sig_handler,
        SIGTSTP => stop_sig_handler,
        SIGTTIN => stop_sig_handler,
        SIGTTOU => stop_sig_handler,
        SIGURG => ign_sig_handler,
        SIGXCPU => core_sig_handler,
        SIGXFSZ => core_sig_handler,
        SIGVTALRM => term_sig_handler,
        SIGPROF => term_sig_handler,
        SIGWINCH => ign_sig_handler,
        SIGIO => term_sig_handler,
        SIGPWR => term_sig_handler,
        SIGSYS => core_sig_handler,
        // The default action for an unhandled real-time signal is to
        // terminate the receiving process.
        _ => term_sig_handler,
    } as *const () as usize;
    handler
}