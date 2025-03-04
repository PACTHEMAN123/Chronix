use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_refmut, translated_str, translated_ref,VirtAddr, VmSpace, VmSpaceHeapExt};
use crate::task::processor::current_trap_cx;
use crate::task::schedule::spawn_user_task;
use crate::task::{
    current_task, current_user_token, exit_current_and_run_next,
};
use crate::trap::TrapContext;
use alloc::{sync::Arc, vec::Vec, string::String};
use log::info;

pub fn sys_exit(exit_code: i32) -> isize {
    //info!("sys_exit: exit_code = {},sepc={}", exit_code,current_trap_cx().sepc);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}


pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

// todo: add add_task, judge whether need to be async
pub fn sys_fork() -> isize {
    //info!("into sys_fork");
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    let task_access = new_task.inner_exclusive_access();
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = task_access.get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    //info!("sys_fork: new_pid = {},user_sp = {:#x}", new_pid,trap_cx.x[2]);
    // add new task to scheduler
    spawn_user_task(new_task);
    new_pid as isize
}

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
        
        let p = task.inner_exclusive_access().trap_cx_ppn.to_kern().get_ref::<TrapContext>().x[2];
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
    // find a child process

    // ---- access current PCB exclusively
    let inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        //info!("sys_waitpid: found zombied child process");
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.vm_space.token(), exit_code_ptr as *mut i32) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

pub async fn sys_yield() -> isize {
    crate::async_utils::yield_now().await;
    0
}
pub fn sys_brk(addr: VirtAddr) -> isize {
    let task = current_task().unwrap();
    let ret  = task.inner_exclusive_access().vm_space.reset_heap_break(addr).0 as isize;
    ret
}
