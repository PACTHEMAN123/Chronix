//! process related syscall

use core::ptr::null;
use core::sync::atomic::Ordering;
use crate::config::PAGE_SIZE;
use crate::fs::fat32::dentry;
use crate::fs::utils::FileReader;
use crate::fs::vfs::dentry::global_find_dentry;
use crate::fs::vfs::DentryState;
use crate::fs::AtFlags;
use crate::fs::{
    vfs::file::open_file,
    OpenFlags,
};
use crate::mm::copy_out;
use crate::mm::{translated_refmut, translated_str, translated_ref};
use crate::processor::context::SumGuard;
use crate::syscall::at_helper;
use crate::task::schedule::spawn_user_task;
use crate::task::{exit_current_and_run_next, INITPROC};
use crate::task::manager::{TaskManager, PROCESS_GROUP_MANAGER, TASK_MANAGER};
use crate::processor::processor::{current_processor, current_task, current_trap_cx, current_user_token, PROCESSORS};
use crate::signal::SigSet;
use crate::utils::{suspend_now, user_path_to_string};
use alloc::string::ToString;
use alloc::{sync::Arc, vec::Vec, string::String};
use hal::addr::{PhysAddrHal, PhysPageNumHal, VirtAddr};
use hal::instruction::{Instruction, InstructionHal};
use hal::pagetable::PageTableHal;
use hal::println;
use hal::trap::{TrapContext, TrapContextHal};
use crate::mm::vm::{KernVmSpaceHal, UserVmSpaceHal};
use log::info;

use super::{SysResult,SysError};

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
        /// CLone_legacy_flag
        const LEGACY_FLAGS = 0xffffffff ;
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
pub fn sys_getpid() -> SysResult {
    // log::info!("[sys_getpid]: in get pid");
    Ok(current_task().unwrap().pid() as isize)
}
/// get the tid of the current thread
pub fn sys_gettid() -> SysResult {
    Ok(current_task().unwrap().tid() as isize)
}

/// exit the current process with the given exit code
pub fn sys_exit(exit_code: i32) -> SysResult {
    exit_current_and_run_next(exit_code);
    Ok(0)
}

/// syscall: set tid address
/// set_tid_address() always returns the caller's thread ID.
/// set_tid_address() always succeeds.
pub fn sys_set_tid_address(tid_ptr: usize) -> SysResult {
    let _sum_guard = SumGuard::new();
    let task = current_task().unwrap().clone();
    task.tid_address().clear_child_tid = Some(tid_ptr);
    Ok(task.tid() as isize)
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
pub fn sys_clone(flags: usize, stack: VirtAddr, parent_tid: VirtAddr, tls: VirtAddr, child_tid: VirtAddr) -> SysResult {
    // info!("[sys_clone]: into clone, stack addr: {:#x}, parent tid: {:?}", stack.0, parent_tid);
    let flags = CloneFlags::from_bits(flags as u64 & !0xff).unwrap();
    let task = current_task().unwrap();
    let new_task = task.fork(flags);
    new_task.get_trap_cx().set_ret_nth(0, 0);
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
    #[cfg(target_arch = "riscv64")]
    if flags.contains(CloneFlags::CHILD_SETTID) {
        unsafe  {
            (child_tid.0 as *mut usize).write_volatile(new_tid);
        }
        new_task.tid_address().set_child_tid = Some(child_tid.0);
    }
    if flags.contains(CloneFlags::CHILD_CLEARTID) {
        new_task.tid_address().clear_child_tid = Some(child_tid.0);
    }
    // todo: more flags...
    if flags.contains(CloneFlags::SETTLS) {
        *new_task.get_trap_cx().tp() = tls.0;
    }
    spawn_user_task(new_task);
    Ok(new_tid as isize)
}


/// execve() executes the program referred to by pathname.  This
/// causes the program that is currently being run by the calling
/// process to be replaced with a new program, with newly initialized
/// stack, heap, and (initialized and uninitialized) data segments.
/// more details, see: https://man7.org/linux/man-pages/man2/execve.2.html
pub async fn sys_execve(pathname: usize, argv: usize, envp: usize) -> SysResult {
    let mut path = user_path_to_string(pathname as *const u8).unwrap();
    let token = current_user_token(&current_processor());
    let mut argv = argv as *const usize;
    let mut envp = envp as *const usize;

    // parse argv
    let mut argv_vec: Vec<String> = Vec::new();
    loop {
        // argv can be specified as null
        if argv == core::ptr::null() {
            break;
        }
        let argv_str_ptr = *translated_ref(token, argv as *const usize);
        if argv_str_ptr == 0 {
            break;
        }
        argv_vec.push(translated_str(token, argv_str_ptr as *const u8));
        unsafe {
            argv = argv.add(1);
        }
    }
    // parse envp
    let mut envp_vec: Vec<String> = Vec::new();
    loop {
        // envp can be specified as null
        if envp == core::ptr::null() {
            break;
        }
        let envp_str_ptr = *translated_ref(token, envp as *const usize);
        if envp_str_ptr == 0 {
            break;
        }
        envp_vec.push(translated_str(token, envp_str_ptr as *const u8));
        unsafe {
            envp = envp.add(1);
        }
    }

    let task = current_task().unwrap().clone();
    // for .sh we will use busybox sh as default
    let dentry = if path.ends_with(".sh") {
        path = "/musl/busybox".to_string();
        argv_vec.insert(0, "busybox".to_string());
        argv_vec.insert(1, "sh".to_string());
        global_find_dentry(&path)?
    } else {
        at_helper(task, AtFlags::AT_FDCWD.bits() as _, pathname as *const u8, AtFlags::empty())?
    };
    // open file
    log::info!("[sys_execve]: try to open file at path {}", dentry.path());
    if dentry.state() != DentryState::NEGATIVE {
        let task = current_task().unwrap();
        let app = dentry.open(OpenFlags::empty()).unwrap();
        let reader = FileReader::new(app.clone());
        let elf = xmas_elf::ElfFile::new(&reader).unwrap();
        task.exec(&elf, Some(app), argv_vec, envp_vec)?;
        Ok(0)
    } else {
        Err(SysError::ENOENT)
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
pub async fn sys_waitpid(pid: isize, exit_code_ptr: usize, option: i32) -> SysResult {
    
    let task = current_task().unwrap().clone();
    log::debug!("[sys_waitpid]: TCB: {}, pid: {pid}, exitcode_ptr: {:x}, option: {option}", task.tid() ,exit_code_ptr);
    let option = WaitOptions::from_bits_truncate(option);
    // todo: now only support for pid == -1 and pid > 0
    // get the all target zombie process
    let res_task = {
        let children = task.children();
        if  children.is_empty() {
            log::debug!("[sys_waitpid]: fail on no child");
            return Err(SysError::ESRCH);
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
            let exit_code = res_task.exit_code();
            log::debug!("[sys_waitpid]: TCB {} first time exit code {}", task.tid() ,exit_code);
            let exit_code = (exit_code & 0xFF) << 8; 
            let exit_code_bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    &exit_code as *const i32 as *const u8,
                    core::mem::size_of::<i32>(),
                )
            };
            copy_out(&mut task.vm_space.lock(), VirtAddr(exit_code_ptr), exit_code_bytes);
        }
        let tid = res_task.tid();
        task.remove_child(tid);
        TASK_MANAGER.remove_task(tid);
        PROCESS_GROUP_MANAGER.remove(&task);
        return Ok(tid as isize);
    } else if option.contains(WaitOptions::WNOHANG) {
        return Ok(0);
    } else {
        log::debug!("[sys_waitpid]: TCB {} waiting for SIGCHLD", task.gettid());
        let (child_pid, exit_code,child_user_time,child_kernel_time) = loop {
            task.set_interruptable();
            let block_sig = task.with_sig_manager(|sig_manager|{
                sig_manager.blocked_sigs
            });
            task.set_wake_up_sigs(!block_sig | SigSet::SIGCHLD);
            suspend_now().await;
            task.set_running();
            
            // todo: missing check if getting the expect signal
            // now check the child one more time
            let si = task.with_mut_sig_manager(|sig_manager|{
                sig_manager.check_pending(SigSet::SIGCHLD)
            });
            if let Some(si) = si {
                log::info!("[sys_waitpid] get signal: {}", si.si_signo);
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
                        child.time_recorder().user_time(),
                        child.time_recorder().kernel_time()
                    );
                }
            }else {
                return Err(SysError::EINTR);
            }
        };
        task.time_recorder().update_child_time((child_user_time,child_kernel_time));
        // write into exit code pointer
        if exit_code_ptr != 0 {
            log::debug!("[sys_waitpid]: TCB {} get child {}, exit code {}", task.tid(), child_pid, exit_code);
            let exit_code = (exit_code & 0xFF) << 8;
            let exit_code_bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    &exit_code as *const i32 as *const u8,
                    core::mem::size_of::<i32>(),
                )
            };
            copy_out(&mut task.vm_space.lock(), VirtAddr(exit_code_ptr), exit_code_bytes);
        }
        task.remove_child(child_pid);
        TASK_MANAGER.remove_task(child_pid);
        //info!("remove task {} from PROCESS_GROUP_MANAGER", task.tid());
        PROCESS_GROUP_MANAGER.remove(&task);
        return Ok(child_pid as isize);
    }
}
/// yield immediatly to another process
pub async fn sys_yield() -> SysResult {
    crate::utils::async_utils::yield_now().await;
    Ok(0)
}
/// change the size of the heap
pub fn sys_brk(addr: VirtAddr) -> SysResult {
    let task = current_task().unwrap();
    let ret  = task.with_mut_vm_space(|vm_space| vm_space.reset_heap_break(addr).0) as isize;
    Ok(ret)
}

/// syscall: get_ppid
pub fn sys_getppid() -> SysResult {
    let task = current_task().unwrap().clone();
    if let Some(parent) = task.parent() {
        let parent = parent.upgrade().unwrap();
        return Ok(parent.pid() as isize);
    } else {
        return Ok(INITPROC.pid() as isize);
    }
}
/// get the process group id of the specified process
pub fn sys_getpgid(pid: usize) -> SysResult {
    log::debug!("[sys_getpgid]: caller pgid: {}, target pid: {}", current_task().unwrap().pgid(), pid);
    if pid == 0 {
        Ok(current_task().unwrap().pgid() as isize)
    }else {
        match TASK_MANAGER.get_task(pid){
            Some(task) => {
                Ok(task.pgid() as isize)
            }
            None => {
                Err(SysError::ESRCH)
            }
        }
    }
}
/// set the process group id of the specified process
pub fn sys_setpgid(pid: usize, pgid: usize) -> SysResult {
    let task =  if pid == 0{
        current_task().unwrap().clone()
    }else {
        TASK_MANAGER.get_task(pid).unwrap()
    };

    if pgid == 0 {
        PROCESS_GROUP_MANAGER.add_group(&task);
    }else {
        if PROCESS_GROUP_MANAGER.get_group(pgid).is_some() {
            PROCESS_GROUP_MANAGER.add_task_to_group(pgid, &task);
        }else {
            PROCESS_GROUP_MANAGER.add_group(&task);
        }
    }
    Ok(0)
}
/// exit_group - exit all threads in a process
pub fn sys_exit_group(exit_code: i32) -> SysResult {
    let task = current_task().unwrap();
    task.with_thread_group(|tg| {
        for thread in tg.iter() {
            // info!("[sys_exit_group]: exit thread {}", thread.tid())
            thread.set_zombie();
        }
    });
    log::debug!("[sys_exit_group]: set exit code {}", (exit_code & 0xFF) << 8);
    task.set_exit_code((exit_code & 0xFF) << 8);
    Ok(0)
}

/// syscall: getuid
/// returns the real user ID of the calling process.
/// These functions are always successful and never modify errno.
/// todo
pub fn sys_getuid() -> SysResult {
    Ok(0)
}

/// syscall: geteuid
/// returns the effective user ID of the calling process.
/// todo
pub fn sys_geteuid() -> SysResult {
    Ok(0)
}

/// syscall: getegid
/// getegid() returns the effective group ID of the calling process.
/// todo
pub fn sys_getegid() -> SysResult {
    Ok(0)
}

///
pub fn sys_setsid() -> SysResult {
    let task = current_task().unwrap();
    Ok(task.pid() as isize)
}
///  long syscall(SYS_clone3, struct clone_args *cl_args, size_t size);
///  glibc provides no wrapper for clone3(), necessitating the
/// use of syscall(2).
pub fn sys_clone3(cl_args_ptr: usize, size: usize) -> SysResult {
    // log::info!("[sys_clone3]: cl_args_ptr: {:x}, size: {}" , cl_args_ptr, size);
    if size > PAGE_SIZE {
        return Err(SysError::E2BIG);
    }
    if size < CLONE_ARGS_SIZE_VER0 {
        return Err(SysError::EINVAL);
    }
    let cl_args = unsafe {
        Instruction::set_sum();
        *(cl_args_ptr as *const CloneArgs)
    };
    let flags = cl_args.flags;
    // log::info!("[sys_clone3]: flags: {:x}", flags);
    let stack = VirtAddr::from(cl_args.stack);
    // log::info!("[sys_clone3]: stack: {:x}", stack.0);
    let parent_tid = VirtAddr::from(cl_args.parent_tid);
    // log::info!("[sys_clone3]: parent_tid: {:x}", parent_tid.0);
    let tls = VirtAddr::from(cl_args.tls);
    // log::info!("[sys_clone3]: tls: {:x}", tls.0);
    let child_tid = VirtAddr::from(cl_args.child_tid);
    // log::info!("[sys_clone3]: child_tid: {:x}", child_tid.0);
    // log::info!("[sys_clone3]: stack_size: {}, set_tid_size: {}, cgroup: {}" , cl_args.stack_size, cl_args.set_tid_size, cl_args.cgroup);
    sys_clone(flags, stack + cl_args.stack_size, parent_tid, tls, child_tid)
}
  
//  * @flags:        Flags for the new process.
//  *                All flags are valid except for CSIGNAL and
//  *                CLONE_DETACHED.
//  * @pidfd:        If CLONE_PIDFD is set, a pidfd will be
//  *                returned in this argument.
//  * @child_tid:    If CLONE_CHILD_SETTID is set, the TID of the
//  *                child process will be returned in the child's
//  *                memory.
//  * @parent_tid:   If CLONE_PARENT_SETTID is set, the TID of
//  *                the child process will be returned in the
//  *                parent's memory.
//  * @exit_signal:  The exit_signal the parent process will be
//  *                sent when the child exits.
//  * @stack:        Specify the location of the stack for the
//  *                child process.
//  *                Note, @stack is expected to point to the
//  *                lowest address. The stack direction will be
//  *                determined by the kernel and set up
//  *                appropriately based on @stack_size.
//  * @stack_size:   The size of the stack for the child process.
//  * @tls:          If CLONE_SETTLS is set, the tls descriptor
//  *                is set to tls.
//  * @set_tid:      Pointer to an array of type *pid_t. The size
//  *                of the array is defined using @set_tid_size.
//  *                This array is used to select PIDs/TIDs for
//  *                newly created processes. The first element in
//  *                this defines the PID in the most nested PID
//  *                namespace. Each additional element in the array
//  *                defines the PID in the parent PID namespace of
//  *                the original PID namespace. If the array has
//  *                less entries than the number of currently
//  *                nested PID namespaces only the PIDs in the
//  *                corresponding namespaces are set.
//  * @set_tid_size: This defines the size of the array referenced
//  *                in @set_tid. This cannot be larger than the
//  *                kernel's limit of nested PID namespaces.
//  * @cgroup:       If CLONE_INTO_CGROUP is specified set this to
//  *                a file descriptor for the cgroup.
/// clone_args structure for clone3()
/// * struct clone_args - arguments for the clone3 syscall
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct CloneArgs {
    pub flags: usize,
    pub pidfd: usize,
    pub child_tid: usize,
    pub parent_tid: usize,
    pub exit_signal: usize,
    pub stack: usize,
    pub stack_size: usize,
    pub tls: usize,
    pub set_tid: usize,
    pub set_tid_size: usize,
    pub cgroup: usize,
}

const  CLONE_ARGS_SIZE_VER0: usize = 64;
const _CLONE_ARGS_SIZE_VER1:usize =  80; /* sizeof second published struct */
const _CLONE_ARGS_SIZE_VER2: usize =  88; /* sizeof third published struct */
