//! misc syscall
#![allow(missing_docs)]

use hal::constant::ConstantsHal;
use hal::instruction::{Instruction, InstructionHal};
use strum::FromRepr;

use crate::syscall::SysError;
use crate::{fs::devfs::urandom::RNG, task::{current_task, manager::TASK_MANAGER}, timer::get_current_time};

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

/// resource adapt from phoenix
#[derive(FromRepr, Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum Resource {
    // Per-process CPU limit, in seconds.
    CPU = 0,
    // Largest file that can be created, in bytes.
    FSIZE = 1,
    // Maximum size of data segment, in bytes.
    DATA = 2,
    // Maximum size of stack segment, in bytes.
    STACK = 3,
    // Largest core file that can be created, in bytes.
    CORE = 4,
    // Largest resident set size, in bytes.
    // This affects swapping; processes that are exceeding their
    // resident set size will be more likely to have physical memory
    // taken from them.
    RSS = 5,
    // Number of processes.
    NPROC = 6,
    // Number of open files.
    NOFILE = 7,
    // Locked-in-memory address space.
    MEMLOCK = 8,
    // Address space limit.
    AS = 9,
    // Maximum number of file locks.
    LOCKS = 10,
    // Maximum number of pending signals.
    SIGPENDING = 11,
    // Maximum bytes in POSIX message queues.
    MSGQUEUE = 12,
    // Maximum nice priority allowed to raise to.
    // Nice levels 19 .. -20 correspond to 0 .. 39
    // values of this resource limit.
    NICE = 13,
    // Maximum realtime priority allowed for non-priviledged
    // processes.
    RTPRIO = 14,
    // Maximum CPU time in microseconds that a process scheduled under a real-time
    // scheduling policy may consume without making a blocking system
    // call before being forcibly descheduled.
    RTTIME = 15,
}

/// Resource Limit
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RLimit {
    /// Soft limit: the kernel enforces for the corresponding resource
    pub rlim_cur: usize,
    /// Hard limit (ceiling for rlim_cur)
    pub rlim_max: usize,
}

pub const RLIM_INFINITY: usize = usize::MAX;

impl RLimit {
    pub fn new(rlim_cur: usize) -> Self {
        Self {
            rlim_cur,
            rlim_max: RLIM_INFINITY,
        }
    }
}


/// syscall: prlimit64
pub fn sys_prlimit64(pid: usize, resource: i32, new_limit: usize, old_limit: usize) -> SysResult {
    
    let task = if pid == 0 {
        current_task().unwrap().clone()
    } else if let Some(t) = TASK_MANAGER.get_task(pid) {
        t.clone()
    } else {
        return Err(SysError::ESRCH);
    };

    let resource = Resource::from_repr(resource).ok_or(SysError::EINVAL)?;

    if old_limit != 0 {
        let limit = match resource {
            Resource::STACK => RLimit {
                rlim_cur: hal::constant::Constant::USER_STACK_SIZE,
                rlim_max: hal::constant::Constant::USER_STACK_SIZE,
            },
            Resource::NOFILE => task.with_fd_table(|table| table.rlimit()),
            r => {
                log::warn!("[sys_prlimit64] get old_limit : unimplemented {r:?}");
                RLimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                }
            }
        };
        unsafe {
            Instruction::set_sum();
            (old_limit as *mut RLimit).write(limit);
        }
    }
    if new_limit != 0 {
        let limit = unsafe {
            Instruction::set_sum();
            (new_limit as *const RLimit).read()
        };
        match resource {
            Resource::NOFILE => {
                log::debug!("[sys_prlimit64] new_limit: {limit:?}");
                task.with_mut_fd_table(|table| table.set_rlimit(limit));
            }
            r => {
                log::warn!("[sys_prlimit64] set new_limit : unimplemented {r:?}");
            }
        }
    }
    Ok(0)
}
