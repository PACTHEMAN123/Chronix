use super::{SysError,SysResult};
use core::sync::atomic::AtomicUsize;

use crate::task::{manager::TASK_MANAGER, task::CpuMask}; 
#[cfg(feature = "smp")]
/// sets the CPU affinity mask of the thread whose ID is pid to the value specified by mask.
pub fn sys_sched_setaffinity(pid: usize, cpusetsize: usize, mask: usize) -> SysResult {
    if cpusetsize < size_of::<CpuMask> () {
        return Err(SysError::EINVAL);
    }
    if let Some(task) = TASK_MANAGER.get_task(pid) {
        if !task.is_leader() {
            return Err(SysError::ESRCH);
        }
        // todo: handle when pid is 0 , which means calling processor is used but now we have opened all the processors
        let mask = mask as *const CpuMask;
        let cpu_mask = unsafe { *mask };
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
    }else {
        return Err(SysError::ESRCH);
    }
    Ok(0)
}

#[cfg(feature = "smp")]
/// writes the affinity mask of the thread whose ID is pid into the cpu_set_t structure pointed to by mask. 
pub fn sys_sched_getaffinity(pid: usize, cpusetusize: usize, mask: usize) -> SysResult {
    use alloc::task;
    let mask = mask as *mut CpuMask;
    if cpusetusize < size_of::<CpuMask> () {
        return Err(SysError::EINVAL);
    }
    if let Some(task) = TASK_MANAGER.get_task(pid) {
        if !task.is_leader() {
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
        unsafe {mask.write_volatile(cpu_mask)};
        Ok(0)
    }else {
        return Err(SysError::ESRCH);
    }
} 