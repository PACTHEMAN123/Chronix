//! signal handlers

use log::*;
use crate::logging;

pub const SIG_ERR: usize = usize::MAX;
pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;


pub fn term_sig_handler(signo: usize) {
    info!("[term_sig_handler]: term sig handler, sig {}", signo);
}

pub fn ign_sig_handler(signo: usize) {
    info!("[ign_sig_handler]: ignore this sig {}", signo);
}

pub fn stop_sig_handler(signo: usize) {
    info!("[stop_sig_handler]: stop sig handler, sig {}", signo);
}