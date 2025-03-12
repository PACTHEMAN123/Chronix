//! process related syscall

use core::sync::atomic::Ordering;
use crate::fs::ext4::{open_file, OpenFlags};
use crate::mm::{translated_refmut, translated_str, translated_ref,VirtAddr, vm::{VmSpace, VmSpaceHeapExt}};
use crate::task::processor::current_trap_cx;
use crate::task::schedule::spawn_user_task;
use crate::task::{
    current_task, current_user_token, exit_current_and_run_next,
};
use crate::trap::TrapContext;
use alloc::{sync::Arc, vec::Vec, string::String};
use log::info;
/// exit the current process with the given exit code
pub fn sys_exit(exit_code: i32) -> isize {
    exit_current_and_run_next(exit_code);
    0
}

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
/// get the pid of the current process
pub fn sys_getpid() -> isize {
    current_task().unwrap().pid() as isize
}
/// get the tid of the current thread
pub fn sys_gettid() -> isize {
    current_task().unwrap().tid() as isize
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
    trap_cx.x[10] = 0;
    //info!("sys_fork: new_pid = {},user_sp = {:#x}", new_pid,trap_cx.x[2]);
    // add new task to scheduler
    spawn_user_task(new_task);
    //info!("sys_fork: complete, new_pid = {}", new_pid);
    new_pid  as isize
}

/// clone a new process/thread/ using clone flags
pub fn sys_clone (flags: usize, stack: VirtAddr,tls: VirtAddr) -> isize {
    let flags = CloneFlags::from_bits(flags as u64 & !0xff).unwrap();
    let task = current_task().unwrap();
    let new_task = task.fork(flags);
    new_task.get_trap_cx().x[10] = 0;
    let new_tid = new_task.tid();

    if !stack.0 == 0 {
        new_task.get_trap_cx().x[2] = stack.0;
    }
    if flags.contains(CloneFlags::SETTLS) {
        new_task.get_trap_cx().x[4] = tls.0;
    }
    spawn_user_task(new_task);
    new_tid as isize
}
/// execute a new program
pub async fn sys_exec(path: usize, args: usize) -> isize {
    let mut args = args as *const usize;
    let token = current_user_token();
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
        
        let p = task.get_trap_cx_ppn_access().to_kern().get_ref::<TrapContext>().x[2];
        // return p because cx.x[10] will be covered with it later
        p as isize
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub async fn sys_waitpid(pid: isize, exit_code_ptr: usize) -> isize {
    //info!("sys_waitpid: pid = {}, exit_code_ptr = {:#x}", pid, exit_code_ptr);
    let task = current_task().unwrap();
    
    let res = {
        let children = task.children();
        if children.is_empty(){
            info!("sys_waitpid: no child process");
        }
        match pid {
            -1 => {
                //info!("wait for any child process");
                children.values()
                .find(|child| child.is_zombie() && child.with_thread_group(|thread_group| thread_group.len() == 1)
                ).cloned()
            }
            0 => {
                //info!("wait for any child process in the same process group of the calling process");
                unimplemented!();
            }
            p if p > 0 => {
                let p = p as usize;
                //info!("wait for a specific child process with pid {}", p);
                if let Some(child) = children.get(&p ) {
                    if child.is_zombie() && child.with_thread_group(|thread_group| thread_group.len() == 1) {
                        Some(child).cloned()
                    }else {
                        None
                    }
                }else {
                    //info!("have no child process with pid {}",p);
                    None
                }
            }
            _p => {
                //info!("wait for any child process in the process group of pid {}", p);
                unimplemented!();
            }
    
        }
    };
    // find a child process
    // ---- access current PCB exclusively
    if let Some(res) = res {
        //info!("now task {} remove child {} and return its exit code {}", task.tid(),res.tid(),res.exit_code.load(Ordering::Relaxed));
        let tid = res.tid();
        task.remove_child(tid);
        let exit_code = res.exit_code.load(Ordering::Relaxed);
        *translated_refmut(task.with_vm_space(|m| m.token()), exit_code_ptr as *mut i32) = exit_code;
        res.tid() as isize
    }  else {
        // todo : if the waiting task isn't zombie yet, then this time this task should do await, until the waiting task do_exit then use SIGHLD to wake up this task.
        // todo signal handling
        -2
    }
    // ---- release current PCB automatically
}
/// yield immediatly to another process
pub async fn sys_yield() -> isize {
    crate::async_utils::yield_now().await;
    0
}
/// change the size of the heap
pub fn sys_brk(addr: VirtAddr) -> isize {
    let task = current_task().unwrap();
    let ret  = task.with_mut_vm_space(|vm_space| vm_space.reset_heap_break(addr).0) as isize;
    ret
}
