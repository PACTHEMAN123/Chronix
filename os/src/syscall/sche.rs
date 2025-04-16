use super::{SysError,SysResult};
use core::sync::atomic::AtomicUsize;

use crate::task::{manager::TASK_MANAGER, task::CpuMask}; 
/// sets the CPU affinity mask of the thread whose ID is pid to the value specified by mask.
pub fn sys_sched_setaffinity(_pid: usize, _cpusetsize: usize, _mask: usize) -> SysResult {
    #[cfg(feature = "smp")]
    if _cpusetsize < size_of::<CpuMask> () {
        return Err(SysError::EINVAL);
    }
    #[cfg(feature = "smp")]
    if let Some(task) = TASK_MANAGER.get_task(_pid) {
        if !task.is_leader() {
            return Err(SysError::ESRCH);
        }
        // todo: handle when pid is 0 , which means calling processor is used but now we have opened all the processors
        let mask = _mask as *const CpuMask;
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


/// writes the affinity mask of the thread whose ID is pid into the cpu_set_t structure pointed to by mask. 
pub fn sys_sched_getaffinity(_pid: usize, cpusetusize: usize, mask: usize) -> SysResult {
    #[allow(unused)]
    use alloc::task;
    let _mask = mask as *mut CpuMask;
    if cpusetusize < size_of::<CpuMask> () {
        return Err(SysError::EINVAL);
    }else {
        #[cfg(feature = "smp")]
    if let Some(task) = TASK_MANAGER.get_task(_pid) {
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
        unsafe {_mask.write_volatile(cpu_mask)};
        }else {
             return Err(SysError::ESRCH);
        }   
        Ok(0)
    }
} 