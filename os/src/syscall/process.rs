//! process related syscall

use core::ptr::null;
use core::sync::atomic::Ordering;
use crate::fs::{
    ext4::open_file,
    OpenFlags,
};
use crate::mm::copy_out;
use crate::mm::{translated_refmut, translated_str, translated_ref};
use crate::processor::context::SumGuard;
use crate::task::schedule::spawn_user_task;
use crate::task::{ exit_current_and_run_next, INITPROC};
use crate::processor::processor::{current_processor, current_task, current_trap_cx, current_user_token, PROCESSORS};
use crate::signal::SigSet;
use crate::utils::suspend_now;
use alloc::{sync::Arc, vec::Vec, string::String};
use hal::addr::{PhysAddrHal, PhysPageNumHal, VirtAddr};
use hal::pagetable::PageTableHal;
use hal::trap::{TrapContext, TrapContextHal};
use hal::vm::{KernVmSpaceHal, UserVmSpaceHal};
use log::info;

bitflags! {
    /// Defined in <bits/sched.h>
    pub struct CloneFlags: u64 {
        /// Set if VM shared between processes.
        const VM = 0x0000100;
        /// Set if fs info shared between processes.
        const FS = 0x0000200;
        /// Set if open files shared between processes.
        const FILES = 0x0000400;
        /// Set if signal handlers shared.
        const SIGHAND = 0x00000800;
        /// Set if a pidfd should be placed in parent.
        const PIDFD = 0x00001000;
        /// Set if we want to have the same parent as the cloner.
        const PARENT = 0x00008000;
        /// Set to add to same thread group.
        const THREAD = 0x00010000;
        /// Set to shared SVID SEM_UNDO semantics.
        const SYSVSEM = 0x00040000;
        /// Set TLS info.
        const SETTLS = 0x00080000;
        /// Store TID in userlevel buffer before MM copy.
        const PARENT_SETTID = 0x00100000;
        /// Register exit futex and memory location to clear.
        const CHILD_CLEARTID = 0x00200000;
        /// Store TID in userlevel buffer in the child.
        const CHILD_SETTID = 0x01000000;
        /// Create clone detached.
        const DETACHED = 0x00400000;
        /// Set if the tracing process can't
        const UNTRACED = 0x00800000;
        /// New cgroup namespace.
        const NEWCGROUP = 0x02000000;
        /// New utsname group.
        const NEWUTS = 0x04000000;
        /// New ipcs.
        const NEWIPC = 0x08000000;
        /// New user namespace.
        const NEWUSER = 0x10000000;
        /// New pid namespace.
        const NEWPID = 0x20000000;
        /// New network namespace.
        const NEWNET = 0x40000000;
        /// Clone I/O context.
        const IO = 0x80000000 ;
    }
}

bitflags! {
    /// Defined in <bits/waitflags.h>.
    pub struct WaitOptions: i32 {
        /// Don't block waiting.
        const WNOHANG = 0x00000001;
        /// Report status of stopped children.
        const WUNTRACED = 0x00000002;
        /// Report continued child.
        const WCONTINUED = 0x00000008;
    }
}

/// get the pid of the current process
pub fn sys_getpid() -> isize {
    current_task().unwrap().pid() as isize
}
/// get the tid of the current thread
pub fn sys_gettid() -> isize {
    current_task().unwrap().tid() as isize
}

/// exit the current process with the given exit code
pub fn sys_exit(exit_code: i32) -> isize {
    exit_current_and_run_next(exit_code);
    0
}

/// fork a new process
pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork(CloneFlags { bits: 0 });
    //info!("complete sys_fork, new_task = {:}",new_task.pid() );
    let new_pid = new_task.pid();
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.set_arg_nth(0, 0);
    //info!("sys_fork: new_pid = {},user_sp = {:#x}", new_pid,trap_cx.x[2]);
    // add new task to scheduler
    spawn_user_task(new_task);
    //info!("sys_fork: complete, new_pid = {}", new_pid);
    new_pid  as isize
}

/// clone a new process/thread/ using clone flags
pub fn sys_clone(flags: usize, stack: VirtAddr, parent_tid: VirtAddr, tls: VirtAddr, child_tid: VirtAddr) -> isize {
    //info!("[sys_clone]: into clone, stack addr: {:#x}", stack.0);
    let flags = CloneFlags::from_bits(flags as u64 & !0xff).unwrap();
    let task = current_task().unwrap();
    let new_task = task.fork(flags);
    new_task.get_trap_cx().set_arg_nth(0, 0);
    let new_tid = new_task.tid();

    // set new stack
    if stack.0 != 0 {
        *new_task.get_trap_cx().sp() = stack.0;
    }

    // set parent tid and child tid
    let _sum_guard = SumGuard::new();
    if flags.contains(CloneFlags::PARENT_SETTID) {
        unsafe {
            (parent_tid.0 as *mut usize).write_volatile(new_tid);
        }
    }
    if flags.contains(CloneFlags::CHILD_SETTID) {
        unsafe  {
            (child_tid.0 as *mut usize).write_volatile(new_tid);
        }
        // todo: write new_tid into child memory(?)
    }
    // todo: more flags...
    if flags.contains(CloneFlags::SETTLS) {
        *new_task.get_trap_cx().tp() = tls.0;
    }
    spawn_user_task(new_task);
    new_tid as isize
}
/// execute a new program
pub async fn sys_exec(path: usize, args: usize) -> isize {
    let mut args = args as *const usize;
    let token = current_user_token(&current_processor());
    let path = translated_str(token, path as *const u8);

    // parse arguments
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args as *const usize);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    // open file
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        
        // let argc = args_vec.len();
        task.exec(all_data.as_slice(), args_vec);
        
        let p = *task.get_trap_cx_ppn_access().start_addr().get_mut::<TrapContext>().sp();
        // return p because cx.x[10] will be covered with it later
        p as isize
    } else {
        -1
    }
}



/// The waitpid() system call suspends execution of the calling thread
/// until a child specified by pid argument has changed state.  By
/// default, waitpid() waits only for terminated children, but this
/// behavior is modifiable via the options argument, as described
/// below.
/// pid < -1 meaning wait for any child process whose process group ID
/// is equal to the absolute value of pid.
/// pid = -1 meaning wait for any child process.
/// pid = 0 meaning wait for any child process whose process group ID
/// is equal to that of the calling process at the time of the call to waitpid().
/// pid > 0 meaning wait for the child whose process ID is equal to the value of pid.
pub async fn sys_waitpid(pid: isize, exit_code_ptr: usize, option: i32) -> isize {
    let task = current_task().unwrap().clone();
    let option = WaitOptions::from_bits_truncate(option);
    // todo: now only support for pid == -1 and pid > 0
    // get the all target zombie process
    let res_task = {
        let children = task.children();
        if  children.is_empty() {
            info!("[sys_waitpid]: fail on no child");
            return -1;
        }
        match pid {
            -1 => {
                children
                .values()
                .find(|c|c.is_zombie() && c.with_thread_group(|tg| tg.len() == 1))
            }
            pid if pid > 0 => {
                if let Some(child) = children.get(&(pid as usize)) {
                    if child.is_zombie() && child.with_thread_group(|tg| tg.len() == 1) {
                        Some(child)
                    } else {
                        None
                    }
                } else {
                    panic!("[sys_waitpid]: no child with pid {}", pid);
                }
            }
            _ => {
                panic!("[sys_waitpid]: not implement");
            }
        }.cloned()
    };

    if let Some(res_task) = res_task {
        res_task.time_recorder().update_child_time(res_task.time_recorder().time_pair());
        if exit_code_ptr != 0 {
            let exit_code = (res_task.exit_code() & 0xFF) << 8; 
            let exit_code_bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    &exit_code as *const i32 as *const u8,
                    core::mem::size_of::<i32>(),
                )
            };
            copy_out(&task.vm_space.lock().get_page_table(), VirtAddr(exit_code_ptr), exit_code_bytes);
        }
        let tid = res_task.tid();
        task.remove_child(tid);
        return tid as isize;
    } else if option.contains(WaitOptions::WNOHANG) {
        return 0;
    } else {
        //info!("[sys_waitpid]: task {} waiting for SIGCHLD", task.gettid());
        let (child_pid, exit_code) = loop {
            task.set_interruptable();
            task.set_wake_up_sigs(SigSet::SIGCHLD);
            suspend_now().await;
            task.set_running();
            
            // todo: missing check if getting the expect signal
            // now check the child one more time
            let children = task.children();
            let child = match pid {
                -1 => {
                    children
                    .values()
                    .find(|c|c.is_zombie() && c.with_thread_group(|tg| tg.len() == 1))
                }
                pid if pid > 0 => {
                    if let Some(child) = children.get(&(pid as usize)) {
                        if child.is_zombie() && child.with_thread_group(|tg| tg.len() == 1) {
                            Some(child)
                        } else {
                            None
                        }
                    } else {
                        panic!("[sys_waitpid]: no child with pid {}", pid);
                    }
                }
                _ => {
                    panic!("[sys_waitpid]: not implement");
                }
            };
            if let Some(child) = child {
                break (
                    child.pid(),
                    child.exit_code(),
                );
            } else {
                panic!("[sys_waitpid] unexpected result");
            }
        };
        // write into exit code pointer
        if exit_code_ptr != 0 {
            let exit_code = (exit_code & 0xFF) << 8;
            let exit_code_bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    &exit_code as *const i32 as *const u8,
                    core::mem::size_of::<i32>(),
                )
            };
            copy_out(&task.vm_space.lock().get_page_table(), VirtAddr(exit_code_ptr), exit_code_bytes);
        }
        task.remove_child(child_pid);
        return child_pid as isize;
    }
}
/// yield immediatly to another process
pub async fn sys_yield() -> isize {
    crate::utils::async_utils::yield_now().await;
    0
}
/// change the size of the heap
pub fn sys_brk(addr: VirtAddr) -> isize {
    let task = current_task().unwrap();
    let ret  = task.with_mut_vm_space(|vm_space| vm_space.reset_heap_break(addr).0) as isize;
    ret
}

/// syscall: get_ppid
pub fn sys_getppid() -> isize {
    let task = current_task().unwrap().clone();
    if let Some(parent) = task.parent() {
        let parent = parent.upgrade().unwrap();
        return parent.pid() as isize;
    } else {
        return INITPROC.pid() as isize;
    }
}
