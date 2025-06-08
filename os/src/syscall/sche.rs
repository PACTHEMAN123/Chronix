use super::{SysError,SysResult};
use core::sync::atomic::AtomicUsize;

use crate::{mm::{UserPtrSendReader, UserPtrSendWriter}, task::{current_task, manager::TASK_MANAGER, task::CpuMask}}; 

/// syscall: 
/// sets the CPU affinity mask of the thread whose ID is pid to the value specified by mask.
/// If pid is zero, then the calling process is used. 
/// The argument cpusetsize is the length (in bytes) of the data pointed to by mask. 
/// Normally this argument would be specified as sizeof(cpu_set_t).
/// (TODO) If the process specified by pid is not currently running on one of the CPUs specified in mask, 
/// then that process is migrated to one of the CPUs specified in mask.
pub fn sys_sched_setaffinity(pid: usize, cpusetsize: usize, mask_ptr: usize) -> SysResult {
    log::info!("sys_sched_setaffinity: pid {pid} cpusetsize {cpusetsize} mask {:#x}", mask_ptr);
    let cur_task = current_task().unwrap().clone();
    if cpusetsize < size_of::<CpuMask>() {
        return Err(SysError::EINVAL);
    } 
    let task = if pid == 0 {
        cur_task.clone()
    } else {
        if let Some(t) = TASK_MANAGER.get_task(pid) {
            t
        } else {
            return Err(SysError::ESRCH)
        }
    };
    if !task.is_leader() {
        log::warn!("task {pid} not leader, should failed to set affinity?");
        // hack the cyclictest
        // return Err(SysError::ESRCH);
    }
    // todo: handle when pid is 0 , which means calling processor is used but now we have opened all the processors
    let mask = UserPtrSendReader::new_const(mask_ptr as *const CpuMask);
    let cpu_mask = *mask.to_ref(&mut cur_task.vm_space.lock()).ok_or(SysError::EFAULT)?;
    let task_cpu_mask = match cpu_mask {
        CpuMask::CPU_ALL => {
            15
        }
        CpuMask::CPU0 => {
            1
        }
        CpuMask::CPU1 => {
            2
        }
        CpuMask::CPU2 => {
            4
        }
        CpuMask::CPU3 => {
            8
        }
        _ => {
            panic!("Invalid cpu mask")
        }
    };
    task.set_cpu_allowed(task_cpu_mask);
    Ok(0)
}


/// syscall: sched_getaffinity
/// sets the CPU affinity mask of the process whose ID is pid to the value specified by mask. 
/// If pid is zero, then the calling process is used. 
/// The argument cpusetsize is the length (in bytes) of the data pointed to by mask. 
/// Normally this argument would be specified as sizeof(cpu_set_t).
/// On success, the raw sched_getaffinity() system call returns 
/// the size (in bytes) of the cpumask_t data type 
/// that is used internally by the kernel to represent the CPU set bit mask.
pub fn sys_sched_getaffinity(pid: usize, cpusetusize: usize, mask_ptr: usize) -> SysResult {
    log::info!("sys_sched_getaffinity pid {pid} cpusetsize {cpusetusize} mask {:#x}", mask_ptr);
    let cur_task = current_task().unwrap().clone();
    let mask = UserPtrSendWriter::new(mask_ptr as *mut CpuMask);
    if cpusetusize < size_of::<CpuMask>() {
        return Err(SysError::EINVAL);
    } 

    let task = if pid == 0 {
        cur_task.clone()
    } else {
        if let Some(t) = TASK_MANAGER.get_task(pid) {
            t
        } else {
            return Err(SysError::ESRCH)
        }
    };
    if !task.is_leader() {
        log::warn!("get task {pid} not leader");
        return Err(SysError::ESRCH);
    }
    let cpu_mask = match task.cpu_allowed() {
        15 => {
            CpuMask::CPU_ALL
        }
        1 => {
            CpuMask::CPU0
        }
        2 => {
            CpuMask::CPU1
        }
        4 => {
            CpuMask::CPU2
        }
        8 => {
            CpuMask::CPU3
        }
        _ => {
            panic!("Invalid cpu mask")
        }
    };
    log::info!("cpu mask {:?}", cpu_mask);
    *mask.to_mut(&mut cur_task.vm_space.lock()).ok_or(SysError::EFAULT)? = cpu_mask;
    Ok(size_of::<CpuMask>() as isize)
}
///
pub fn sys_sched_setscheduler() -> SysResult {
    log::warn!("[sys_sched_setscheduler] unimplemented");
    Ok(0)
}
///
pub fn sys_sched_getscheduler() -> SysResult {
    log::warn!("[sys_sched_getscheduler] unimplemented");
    Ok(0)
}
/// 
pub fn sys_sched_getparam() -> SysResult {
    log::warn!("[sys_sched_getparam] unimplemented");
    Ok(0)
}