use core::arch::asm;

use crate::{SignalAction, TimeVal};

const SYSCALL_DUP: usize = 24;
const SYSCALL_CHDIR: usize = 49;
const SYSCALL_OPENAT: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_SIGACTION: usize = 134;
const SYSCALL_SIGPROCMASK: usize = 135;
const SYSCALL_SIGRETURN: usize = 139;
const SYSCALL_REBOOT: usize = 142;
const SYSCALL_GETTIMEOFDAY: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_SOCKET: usize = 198;
const SYSCALL_BIND: usize = 200;
const SYSCALL_LISTEN: usize = 201;
const SYSCALL_ACCEPT: usize = 202;
const SYSCALL_CONNECT: usize = 203;
const SYSCALL_SENDTO: usize = 206;
const SYSCALL_RECVFROM: usize = 207;
const SYSCALL_BRK: usize = 214;
const SYSCALL_CLONE: usize = 220;
const SYSCALL_EXECVE: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_MREMAP: usize = 216;
const SYSCALL_MMAP: usize = 222;

#[cfg(target_arch="riscv64")]
fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a3") args[3],
            in("a4") args[4],
            in("a5") args[5],
            in("a7") id
        );
    }
    ret
}

#[cfg(target_arch="loongarch64")]
fn syscall(id: usize, args: [usize; 6]) -> isize {
    use core::arch;

    let mut ret: isize;
    unsafe {
        asm!(
            "syscall 0",
            inlateout("$a0") args[0] => ret,
            in("$a1") args[1],
            in("$a2") args[2],
            in("$a3") args[3],
            in("$a4") args[4],
            in("$a5") args[5],
            in("$a7") id
        );
    }
    ret
}

pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0,0,0,0])
}

pub fn sys_chdir(path: *const u8) -> isize {
    syscall(SYSCALL_CHDIR, [path as usize, 0, 0, 0, 0, 0])
}

pub fn sys_openat(dirfd: isize, path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPENAT, [dirfd as usize, path.as_ptr() as usize, flags as usize, 0, 0, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0,0,0,0])
}

pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0,0,0,0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len(), 0, 0, 0],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8], len: usize) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, len, 0, 0, 0])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0,0,0,0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0,0,0,0])
}

pub fn sys_kill(pid: usize, signal: i32) -> isize {
    syscall(SYSCALL_KILL, [pid, signal as usize, 0,0,0,0])
}

pub fn sys_get_time_of_day(tv: &mut TimeVal) -> isize {
    syscall(SYSCALL_GETTIMEOFDAY, [tv as *mut _ as usize, 0, 0,0,0,0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0, 0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_CLONE, [0, 0, 0, 0, 0, 0])
}
pub fn sys_clone(flags: usize, stack: usize, tls: usize) -> isize {
    syscall(
        SYSCALL_CLONE,
        [flags, stack, tls, 0, 0, 0], 
    )
}

pub fn sys_exec(path: &str, args: &[*const u8]) -> isize {
    syscall(
        SYSCALL_EXECVE,
        [path.as_ptr() as usize, args.as_ptr() as usize, 0,0,0,0],
    )
}

pub fn sys_execve(path: *const u8, argv: usize, envp: usize) -> isize {
    syscall(
        SYSCALL_EXECVE,
        [path as usize, argv, envp, 0, 0, 0]
    )
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0, 0, 0, 0])
}

pub fn sys_sigaction(
    signum: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction,
) -> isize {
    syscall(
        SYSCALL_SIGACTION,
        [signum as usize, action as usize, old_action as usize, 0, 0, 0],
    )
    /*
    syscall(
        SYSCALL_SIGACTION,
        [
            signum as usize,
            action.map_or(0, |r| r as *const _ as usize),
            old_action.map_or(0, |r| r as *mut _ as usize),
        ],
    )
    */
}

pub fn sys_sigprocmask(mask: u32) -> isize {
    syscall(SYSCALL_SIGPROCMASK, [mask as usize, 0, 0, 0, 0, 0])
}

pub fn sys_sigreturn() -> isize {
    syscall(SYSCALL_SIGRETURN, [0, 0, 0, 0, 0, 0])
}

pub fn sys_brk(new_brk: usize) -> isize {
    syscall(SYSCALL_BRK, [new_brk, 0, 0, 0, 0, 0])
}

pub fn sys_socket(domain: usize, sock_type: usize, protocal: usize) -> isize {
    syscall(SYSCALL_SOCKET, [domain,sock_type,protocal, 0, 0, 0])
}

pub fn sys_bind(fd: usize, addr: *const u8, addr_len: u32) -> isize {
    syscall(SYSCALL_BIND, [fd, addr as usize, addr_len as usize, 0, 0, 0])
}

pub fn sys_listen(fd: usize, backlog: i32) -> isize {
    syscall(SYSCALL_LISTEN, [fd, backlog as usize, 0, 0, 0, 0])
}

pub fn sys_connect(fd: usize, addr: *const u8, addr_len: u32) -> isize {
    syscall(SYSCALL_CONNECT, [fd, addr as usize, addr_len as usize, 0, 0, 0])
}

pub fn sys_accept(fd: usize, addr: *mut u8, addr_len: *mut u32) -> isize {
    syscall(SYSCALL_ACCEPT, [fd, addr as usize, addr_len as usize, 0, 0, 0])
}
pub fn sys_sendto(sockfd: i32, buf: *const u8, len: usize, flags: i32, dest_addr: *const u8, addrlen: u32) -> isize{
    println!("addrlen is {}", addrlen);
    syscall(SYSCALL_SENDTO, [sockfd as usize, buf as usize, len, flags as usize, dest_addr as usize, addrlen as usize])
}

pub fn sys_recvfrom(sockfd: i32, buf: *mut u8, len: usize, flags: i32, src_addr: *mut u8, addrlen: *mut u32) -> isize{
    syscall(SYSCALL_RECVFROM, [sockfd as usize, buf as usize, len, flags as usize, src_addr as usize, addrlen as usize])
}

pub fn sys_mmap(addr: usize, len: usize, prot: i32, flags: i32, fd: usize, offset: usize) -> isize {
    syscall(SYSCALL_MMAP, [addr, len, prot as _, flags as _, fd, offset])
}

pub fn sys_mremap(old_addr: usize, old_size: usize, new_size: usize, flags: i32, new_addr:usize) -> isize {
    syscall(SYSCALL_MREMAP, [old_addr, old_size, new_size, flags as _, new_addr, 0])
}

pub fn sys_shutdown(magic1: i32, magic2: i32, cmd: u32, args: usize) -> isize {
    syscall(SYSCALL_REBOOT, [magic1 as _, magic2 as _, cmd as _, args, 0, 0])
}