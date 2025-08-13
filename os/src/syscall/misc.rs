//! misc syscall
#![allow(missing_docs)]

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use hal::constant::ConstantsHal;
use hal::instruction::{Instruction, InstructionHal};
use strum::FromRepr;
use lazy_static::lazy_static;

use crate::mm::{UserPtrRaw, UserSliceRaw};
use crate::sync::mutex::SpinNoIrqLock;
use crate::syscall::SysError;
use crate::{fs::devfs::urandom::RNG, task::{current_task, manager::TASK_MANAGER}, timer::{get_current_time,ffi::TimeVal}};

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
    // unsafe {
    //     Instruction::set_sum();
    //     (info as *mut Sysinfo).write_volatile(sysinfo);
    // }
    let task = current_task().unwrap();
    let info = UserPtrRaw::new(info as *mut Sysinfo)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    info.write(sysinfo);
    Ok(0)
}

/// syscall: get random
pub fn sys_getrandom(buf: usize, len: usize, _flags: usize) -> SysResult {
    // let mut buf_slice = unsafe {
    //     Instruction::set_sum();
    //     core::slice::from_raw_parts_mut(buf as *mut u8, len)
    // };
    log::info!("getrandom: buf: {:?}, len: {:?}, flags: {:?}", buf, len, _flags);
    let task = current_task().unwrap();
    let buf = UserSliceRaw::new(buf as *mut u8, len)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    
    let buf_slice = buf.to_mut();
    RNG.lock().fill_buf(buf_slice);
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
        // unsafe {
        //     Instruction::set_sum();
        //     (old_limit as *mut RLimit).write(limit);
        // }
        let old_limit = UserPtrRaw::new(old_limit as *mut RLimit)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        old_limit.write(limit);
    }
    if new_limit != 0 {
        // let limit = unsafe {
        //     Instruction::set_sum();
        //     (new_limit as *const RLimit).read()
        // };
        let limit = *UserPtrRaw::new(new_limit as *const RLimit)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?
            .to_ref();
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

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Rusage {
    /// user CPU time used
    pub ru_utime: TimeVal,
    /// system CPU time used
    pub ru_stime: TimeVal,
    /// maximum resident set size
    pub ru_maxrss: usize,
    /// integral shared memory size
    pub ru_ixrss: usize,
    /// integral unshared data size
    pub ru_idrss: usize,
    /// integral unshared stack size
    pub ru_isrss: usize,
    /// page reclaims (soft page faults)
    pub ru_minflt: usize,
    /// page faults (hard page faults)
    pub ru_majflt: usize,
    /// swaps
    pub ru_nswap: usize,
    /// block input operations
    pub ru_inblock: usize,
    /// block output operations
    pub ru_oublock: usize,
    /// IPC messages sent
    pub ru_msgsnd: usize,
    /// IPC messages received
    pub ru_msgrcv: usize,
    /// signals received
    pub ru_nsignals: usize,
    /// voluntary context switches
    pub ru_nvcsw: usize,
    /// involuntary context switches
    pub ru_nivcsw: usize,
}

/// getrusage - get resource usage
/// getrusage() returns resource usage measures for who, which can be
///    one of the following:
///    RUSAGE_SELF
///           Return resource usage statistics for the calling process,
///           which is the sum of resources used by all threads in the
///           process.

///    RUSAGE_CHILDREN
///           Return resource usage statistics for all children of the
///           calling process that have terminated and been waited for.
///           These statistics will include the resources used by
///           grandchildren, and further removed descendants, if all of
///           the intervening descendants waited on their terminated
///           children.

///    RUSAGE_THREAD (since Linux 2.6.26)
///           Return resource usage statistics for the calling thread.
///           The _GNU_SOURCE feature test macro must be defined (before
///           including any header file) in order to obtain the
///           definition of this constant from <sys/resource.h>.
const RUSAGE_SELF: i32 = 0;
const RUSAGE_CHILDREN: i32 = -1;
const RUSAGE_THREAD: i32 = 1;

/// syscall: getrusage
pub fn sys_getrusage(who: i32, usage: usize) -> SysResult {
    let task = current_task().unwrap();
    let mut res = Rusage::default();
    match who {
        RUSAGE_SELF => {
            let (utime, stime) = task.time_recorder().time_pair();
            res.ru_utime = utime.into();
            res.ru_stime = stime.into();
            // unsafe {
            //     let usage_ptr = usage as *mut Rusage;
            //     usage_ptr.write(res);
            // }
            let usage_ptr = UserPtrRaw::new(usage as *mut Rusage)
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
            usage_ptr.write(res);
        }
        RUSAGE_CHILDREN => {
            let (c_utime, c_stime) = task.time_recorder().child_time_pair();
            res.ru_utime = c_utime.into();
            res.ru_stime = c_stime.into();
            // unsafe {
            //     let usage_ptr = usage as *mut Rusage;
            //     usage_ptr.write(res);
            // }
            let usage_ptr = UserPtrRaw::new(usage as *mut Rusage)
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
            usage_ptr.write(res);
        }
        RUSAGE_THREAD => {
            let (utime, stime) = task.time_recorder().time_pair();
            res.ru_utime = utime.into();
            res.ru_stime = stime.into();
            // unsafe {
            //     let usage_ptr = usage as *mut Rusage;
            //     usage_ptr.write(res);
            // }
            let usage_ptr = UserPtrRaw::new(usage as *mut Rusage)
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
            usage_ptr.write(res);
        }
        _ => {
            return Err(SysError::EINVAL);
        }
    } 
    Ok(0)
}


// Defined in <sys/utsname.h>.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UtsName {
    /// Name of the implementation of the operating system.
    pub sysname: [u8; 65],
    /// Name of this node on the network.
    pub nodename: [u8; 65],
    /// Current release level of this implementation.
    pub release: [u8; 65],
    /// Current version level of this release.
    pub version: [u8; 65],
    /// Name of the hardware type the system is running on.
    pub machine: [u8; 65],
    /// Name of the domain of this node on the network.
    pub domainname: [u8; 65],
}

impl UtsName {
    fn from_str(info: &str) -> [u8; 65] {
        let mut data: [u8; 65] = [0; 65];
        data[..info.len()].copy_from_slice(info.as_bytes());
        data
    }
}

pub struct UtsManager {
    /// Name of the implementation of the operating system.
    pub sysname: String,
    /// Name of this node on the network.
    pub nodename: String,
    /// Current release level of this implementation.
    pub release: String,
    /// Current version level of this release.
    pub version: String,
    /// Name of the hardware type the system is running on.
    pub machine: String,
    /// Name of the domain of this node on the network.
    pub domainname: String,
}

impl UtsManager {
    pub fn new() -> Self {
        Self {
            sysname: "Linux".to_string(),
            nodename: "Linux".to_string(),
            release: "5.19.0-42-generic".to_string(),
            version: "#43~22.04.1-Ubuntu SMP PREEMPT_DYNAMIC Fri Apr 21 16:51:08 UTC 2".to_string(),
            machine: "RISC-V SiFive Freedom U740 SoC".to_string(),
            domainname: "localhost".to_string()
        }
    }

    pub fn get_utsname(&self) -> UtsName {
        UtsName {
            sysname: UtsName::from_str(&self.sysname),
            nodename: UtsName::from_str(&self.nodename),
            release: UtsName::from_str(&self.release),
            version: UtsName::from_str(&self.version),
            machine: UtsName::from_str(&self.machine),
            domainname: UtsName::from_str(&self.domainname),
        }
    }

    pub fn set_nodename(&mut self, nodename: &str) {
        self.nodename = nodename.to_string()
    }

    pub fn set_domainname(&mut self, domainname: &str) {
        self.domainname = domainname.to_string()
    }
}

lazy_static! {
    pub static ref UTS: SpinNoIrqLock<UtsManager> = SpinNoIrqLock::new(UtsManager::new());
}

/// syscall uname
pub fn sys_uname(uname_buf: usize) -> SysResult {
    let uname = UTS.lock().get_utsname();
    // let uname_ptr = uname_buf as *mut UtsName;
    let task = current_task().unwrap();
    let uname_ptr = UserPtrRaw::new(uname_buf as *mut UtsName)
        .ensure_write(&mut task.vm_space.lock())
        .ok_or(SysError::EFAULT)?;
    uname_ptr.write(uname);
    Ok(0)
}

pub fn sys_getdomainname(buf_ptr: usize, len: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let buf = UserSliceRaw::new(buf_ptr as *mut u8, len)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let domainname = UTS.lock().get_utsname().domainname;
    buf.to_mut().copy_from_slice(&domainname[..len]);
    Ok(0)
}

pub fn sys_setdomainname(buf_ptr: usize, len: usize) -> SysResult {
    log::info!("buf_ptr {:#x}, len {}", buf_ptr, len);
    if (len as isize) < 0 || (len as isize) > 64 {
        return Err(SysError::EINVAL);
    }
    if buf_ptr == 0 {
        return Err(SysError::EFAULT)
    }
    let task = current_task().unwrap().clone();
    let buf = UserSliceRaw::new(buf_ptr as *const u8, len)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let domainname = String::from_utf8(buf.to_ref().to_vec()).map_err(|_| SysError::EINVAL)?;
    UTS.lock().set_domainname(&domainname);
    Ok(0)
}