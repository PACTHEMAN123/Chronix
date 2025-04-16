//! misc syscall
#![allow(missing_docs)]

use hal::instruction::{Instruction, InstructionHal};

use crate::{fs::devfs::urandom::RNG, timer::get_current_time};

use super::SysResult;


pub const SYSINFO_SIZE: usize = size_of::<Sysinfo>();

const _F_SIZE: usize = 20 - 2 * size_of::<u64>() - size_of::<u32>();

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Sysinfo {
    /// Seconds since boot
    pub uptime: i64,
    /// 1, 5, and 15 minute load averages
    pub loads: [u64; 3],
    /// Total usable main memory size
    pub totalram: u64,
    /// Available memory size
    pub freeram: u64,
    /// Amount of shared memory
    pub sharedram: u64,
    /// Memory used by buffers
    pub bufferram: u64,
    /// Total swap space size
    pub totalswap: u64,
    /// swap space still available
    pub freeswap: u64,
    /// Number of current processes
    pub procs: u16,
    /// Explicit padding for m68k
    pub pad: u16,
    /// Total high memory size
    pub totalhigh: u64,
    /// Available high memory size
    pub freehigh: u64,
    /// Memory unit size in bytes
    pub mem_uint: u32,
    /// Padding: libc5 uses this..
    pub _f: [u8; _F_SIZE],
}

/// syscall: sysinfo
/// TODO: unimlement
pub fn sys_sysinfo(info: usize) -> SysResult {
    let sysinfo = Sysinfo {
        uptime: get_current_time() as i64,
        loads: [0; 3],
        totalram: 0,
        freeram: 0,
        sharedram: 0,
        bufferram: 0,
        totalswap: 0,
        freeswap: 0,
        procs: 0,
        pad: 0,
        totalhigh: 0,
        freehigh: 0,
        mem_uint: 0,
        _f: [0; _F_SIZE],
    };
    unsafe {
        Instruction::set_sum();
        (info as *mut Sysinfo).write_volatile(sysinfo);
    }
    Ok(0)
}

/// syscall: get random
pub fn sys_getrandom(buf: usize, len: usize, _flags: usize) -> SysResult {
    let mut buf_slice = unsafe {
        Instruction::set_sum();
        core::slice::from_raw_parts_mut(buf as *mut u8, len)
    };

    RNG.lock().fill_buf(&mut buf_slice);
    Ok(buf_slice.len() as isize)
}
