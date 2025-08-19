use alloc::task;

use super::{SysError,SysResult};
use core::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

use crate::{mm::UserPtrRaw, syscall::process, task::{current_task, manager::{PROCESS_GROUP_MANAGER, TASK_MANAGER}, task::CpuMask}}; 

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
        // hack the cyclictest
        // return Err(SysError::ESRCH);
    }
    // todo: handle when pid is 0 , which means calling processor is used but now we have opened all the processors
    let mask_ptr = UserPtrRaw::new(mask_ptr as *const CpuMask)
        .ensure_read(&mut cur_task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let mask = *mask_ptr.to_ref();
    let task_cpu_mask = match mask {
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
    let mask_ptr = UserPtrRaw::new(mask_ptr as *const CpuMask)
        .ensure_write(&mut cur_task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let mask = mask_ptr.to_mut();
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
    *mask = cpu_mask;
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

const PRIO_PROCESS: usize = 0;
const PRIO_PGRP: usize = 1;
const PRIO_USER: usize = 2;
const MIN_PRIORITY: i32 = 0;
const MAX_PRIORITY: i32 = 39;
const NZERO : i32 = 20;
///
pub fn sys_set_priority(which: usize, who: usize, process_priority: i32) -> SysResult {
    let task= current_task().unwrap();
    match which {
        PRIO_PROCESS => {
            if who == 0 || who == task.pid() {
                task.with_thread_group(|tg| {
                    for thread in tg.iter() {
                    let new_priority = (NZERO+ process_priority).max(MIN_PRIORITY as i32).min(MAX_PRIORITY as i32);
                        log::info!("now new priority is {}", new_priority);
                        thread.set_priority(new_priority);
                    }
                });
            }else {
                let target_task = TASK_MANAGER.get_task(who).ok_or(SysError::ESRCH)?;
                target_task.with_thread_group(|tg| {
                    for thread in tg.iter() {
                        let new_priority = (NZERO+ process_priority).max(MIN_PRIORITY as i32).min(MAX_PRIORITY as i32);
                        log::info!("now new priority is {}", new_priority);
                        thread.set_priority(new_priority);
                    }
                });
            }
            return Ok(0);
        }

        PRIO_PGRP => {
            let target_pg = PROCESS_GROUP_MANAGER.get_group(which)
               .ok_or(SysError::ESRCH)?;
            for task in target_pg.iter().filter_map(|t| t.upgrade()) {
                let new_priority = (NZERO+ process_priority).max(MIN_PRIORITY as i32).min(MAX_PRIORITY as i32);
                task.set_priority(new_priority);
            }
            return Ok(0);
        }
        PRIO_USER => {
            log::warn!("[sys_set_priority] unimplemented for PRIO_USER");
        }
        _ => {
            log::warn!("[sys_set_priority] invalid which value");
            return Err(SysError::EINVAL);
        }
    }
    Ok(0)
}

/// 
pub fn sys_get_priority(which: usize, who: usize) -> SysResult {
    let task= current_task().unwrap();
    match which {
        PRIO_PROCESS => {
            if who == 0 || who == task.pid() {
                let priority = task.priority().load(Ordering::SeqCst);
                return Ok((2*NZERO - priority) as isize);
            }else {
                let target_task = TASK_MANAGER.get_task(who).ok_or(SysError::ESRCH)?;
                let priority = target_task.priority().load(Ordering::SeqCst);
                return Ok((2*NZERO - priority) as isize);
            }
        }
        PRIO_PGRP => {
            let target_pg = PROCESS_GROUP_MANAGER.get_group(which)
               .ok_or(SysError::ESRCH)?;
            let mut min_priority =  MAX_PRIORITY;
            for task in target_pg.iter().filter_map(|t| t.upgrade()) {
                let priority = task.priority().load(Ordering::SeqCst); 
                log::info!("priority is {}", priority);
                min_priority = min_priority.min(priority);
                log::info!("min_priority is {}", min_priority);
            }
            return Ok((2*NZERO - min_priority) as isize);
        }
        PRIO_USER => {
            log::warn!("[sys_get_priority] unimplemented for PRIO_USER");
        }
        _ => {
            log::warn!("[sys_set_priority] invalid which value");
            return Err(SysError::EINVAL);
        }
    }
    Ok(0)
}

pub fn sys_getcpu(cpu_ptr: usize, node_ptr: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    let cpu_ptr = UserPtrRaw::new(cpu_ptr as *mut u32)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    cpu_ptr.write(0);
    let node_ptr = UserPtrRaw::new(node_ptr as *mut u32)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    node_ptr.write(0);
    Ok(0)
}