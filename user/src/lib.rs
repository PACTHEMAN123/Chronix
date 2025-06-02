#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use alloc::{ffi::CString, vec::Vec};
use buddy_system_allocator::LockedHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 32768;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
#[naked]
pub unsafe extern "C" fn _start() {
    #[cfg(target_arch="riscv64")]
    core::arch::naked_asm!(
        "
        mv a0, sp
        jal ra, _rust_start
        "
    );
    #[cfg(target_arch="loongarch64")]
    core::arch::naked_asm!(
        "
        move $a0, $sp
        bl _rust_start
        "
    );
}

#[no_mangle]
pub fn _rust_start(p: *const usize) -> ! {

    let argc = unsafe { p.read_volatile() };
    let argv = unsafe { p.add(1) as usize };
    
    unsafe {
        #[allow(static_mut_refs)]
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    
    let mut v: Vec<&'static str> = Vec::new();
    for i in 0..argc {
        let str_start =
            unsafe {
                ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() 
            };
        let len = (0usize..)
            .find(|i| unsafe { ((str_start + *i) as *const u8).read_volatile() == 0 })
            .unwrap();
        v.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(str_start as *const u8, len)
            })
            .unwrap(),
        );
    }
    exit(main(v.as_slice()));
}

#[linkage = "weak"]
#[no_mangle]
fn main(_args: &[&str]) -> i32 {
    panic!("Cannot find main!");
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
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

pub fn thread_create(flags:CloneFlags) -> isize {
    let mut stack: [usize;1024] = [0;1024];
    sys_clone(flags.bits() as _, stack.as_mut_ptr() as usize, 0)
}
pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}

pub fn chdir(path: &str) -> isize {
    sys_chdir(path.as_ptr() as *const u8)
}

pub const AT_FDCWD: isize = -100;
pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_openat(AT_FDCWD, path, flags.bits)
}
pub fn close(fd: usize) -> isize {
    sys_close(fd)
}
pub fn pipe(pipe_fd: &mut [usize]) -> isize {
    sys_pipe(pipe_fd)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn write(fd: usize, buf: &[u8], len: usize) -> isize {
    sys_write(fd, buf, len)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}
pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time_ms() -> isize {
    let mut tv: TimeVal = TimeVal { sec: 0, usec: 0 };
    let ret = sys_get_time_of_day(&mut tv);
    if ret < 0 {
        return ret;
    }
    return (tv.sec*1000 + tv.usec/1000) as isize;
}

pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn clone(flags: usize, stack: usize, tls: usize) -> isize {
    sys_clone(flags, stack, tls)
}
pub fn exec(path: &str, args: &[*const u8]) -> isize {
    sys_exec(path, args)
}

pub fn execve(path: &str, argv: &[&str], envp: &[&str]) -> isize {
    let path = CString::new(path).unwrap();
    let argv: Vec<_> = argv.iter().map(|s| CString::new(*s).unwrap()).collect();
    let envp: Vec<_> = envp.iter().map(|s| CString::new(*s).unwrap()).collect();
    let mut argv = argv.iter().map(|s| s.as_ptr() as usize).collect::<Vec<_>>();
    let mut envp = envp.iter().map(|s| s.as_ptr() as usize).collect::<Vec<_>>();
    argv.push(0);
    envp.push(0);
    sys_execve(path.as_ptr() as *const u8, argv.as_ptr() as usize, envp.as_ptr() as usize)
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid_nb(pid: isize, exit_code: &mut i32) -> isize {
    sys_waitpid(pid as isize, exit_code as *mut _)
}

pub fn sleep(period_ms: usize) {
    let start = get_time_ms();
    while get_time_ms() < start + period_ms as isize {
        sys_yield();
    }
}

/// Action for a signal
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct SignalAction {
    pub handler: usize,
    pub mask: SignalFlags,
}

impl Default for SignalAction {
    fn default() -> Self {
        Self {
            handler: 0,
            mask: SignalFlags::empty(),
        }
    }
}

pub const SIGDEF: i32 = 0; // Default signal handling
pub const SIGHUP: i32 = 1;
pub const SIGINT: i32 = 2;
pub const SIGQUIT: i32 = 3;
pub const SIGILL: i32 = 4;
pub const SIGTRAP: i32 = 5;
pub const SIGABRT: i32 = 6;
pub const SIGBUS: i32 = 7;
pub const SIGFPE: i32 = 8;
pub const SIGKILL: i32 = 9;
pub const SIGUSR1: i32 = 10;
pub const SIGSEGV: i32 = 11;
pub const SIGUSR2: i32 = 12;
pub const SIGPIPE: i32 = 13;
pub const SIGALRM: i32 = 14;
pub const SIGTERM: i32 = 15;
pub const SIGSTKFLT: i32 = 16;
pub const SIGCHLD: i32 = 17;
pub const SIGCONT: i32 = 18;
pub const SIGSTOP: i32 = 19;
pub const SIGTSTP: i32 = 20;
pub const SIGTTIN: i32 = 21;
pub const SIGTTOU: i32 = 22;
pub const SIGURG: i32 = 23;
pub const SIGXCPU: i32 = 24;
pub const SIGXFSZ: i32 = 25;
pub const SIGVTALRM: i32 = 26;
pub const SIGPROF: i32 = 27;
pub const SIGWINCH: i32 = 28;
pub const SIGIO: i32 = 29;
pub const SIGPWR: i32 = 30;
pub const SIGSYS: i32 = 31;

bitflags! {
    pub struct SignalFlags: i32 {
        const SIGDEF = 1; // Default signal handling
        const SIGHUP = 1 << 1;
        const SIGINT = 1 << 2;
        const SIGQUIT = 1 << 3;
        const SIGILL = 1 << 4;
        const SIGTRAP = 1 << 5;
        const SIGABRT = 1 << 6;
        const SIGBUS = 1 << 7;
        const SIGFPE = 1 << 8;
        const SIGKILL = 1 << 9;
        const SIGUSR1 = 1 << 10;
        const SIGSEGV = 1 << 11;
        const SIGUSR2 = 1 << 12;
        const SIGPIPE = 1 << 13;
        const SIGALRM = 1 << 14;
        const SIGTERM = 1 << 15;
        const SIGSTKFLT = 1 << 16;
        const SIGCHLD = 1 << 17;
        const SIGCONT = 1 << 18;
        const SIGSTOP = 1 << 19;
        const SIGTSTP = 1 << 20;
        const SIGTTIN = 1 << 21;
        const SIGTTOU = 1 << 22;
        const SIGURG = 1 << 23;
        const SIGXCPU = 1 << 24;
        const SIGXFSZ = 1 << 25;
        const SIGVTALRM = 1 << 26;
        const SIGPROF = 1 << 27;
        const SIGWINCH = 1 << 28;
        const SIGIO = 1 << 29;
        const SIGPWR = 1 << 30;
        const SIGSYS = 1 << 31;
    }
}

pub fn kill(pid: isize, signum: i32) -> isize {
    sys_kill(pid as usize, signum)
}

pub fn sigaction(
    signum: i32,
    action: Option<&SignalAction>,
    old_action: Option<&mut SignalAction>,
) -> isize {
    sys_sigaction(
        signum,
        action.map_or(core::ptr::null(), |a| a),
        old_action.map_or(core::ptr::null_mut(), |a| a),
    )
}

pub fn sigprocmask(mask: u32) -> isize {
    sys_sigprocmask(mask)
}

pub fn sigreturn() -> isize {
    sys_sigreturn()
}

pub fn brk(new_brk: usize) -> isize {
    sys_brk(new_brk)
}

#[repr(C)]
pub struct SockaddrIn {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub sin_zero: [u8; 8],
}
const AF_INET: u16 = 2;
impl SockaddrIn {
    pub fn new(ip: u32, port: u16) -> Self {
        SockaddrIn {
            sin_family: AF_INET,
            sin_port: port,
            sin_addr: ip,
            sin_zero: [0; 8],
        }
    }
}
pub fn parse_ipv4(s: &str) -> Option<u32> {
    let mut addr: u32 = 0;
    for (i, octet) in s.split('.').enumerate() {
        let byte: u8 = octet.parse().ok()?;
        addr |= (byte as u32) << (24 - 8*i);
    }
    Some(addr)
}
pub fn socket(domain: i32, sock_type: i32, protocol: i32) -> isize {
    sys_socket(domain as usize, sock_type as usize, protocol as usize)
}

pub fn bind(fd: usize, addr: *const SockaddrIn, addr_len: u32) -> isize {
    sys_bind(fd, addr as *const _ as *const u8, addr_len)
}

pub fn listen (fd: usize, backlog: i32) -> isize {
    sys_listen(fd, backlog )
}

pub fn accept (fd: usize, addr: *mut SockaddrIn, addr_len: *mut u32) -> isize {
    sys_accept(fd, addr as *mut _ as *mut u8, addr_len)
}

pub fn connect(fd: usize, addr: *const SockaddrIn, addr_len: u32) -> isize {
    sys_connect(fd, addr as *const _ as *const u8, addr_len)
}

pub fn sendto(fd: usize, buf: &[u8], len: usize, flags: i32, addr: *const SockaddrIn, addr_len: u32) -> isize {
    sys_sendto(fd as i32, buf.as_ptr() , len, flags, addr as *const _ , addr_len)
}

pub fn recvfrom(fd: usize, buf: &mut [u8], len: usize, flags: i32, addr: *mut SockaddrIn, addr_len: *mut u32) -> isize {
    sys_recvfrom(fd as i32, buf.as_ptr() as *mut u8, len, flags, addr as *mut _ , addr_len)
}

bitflags! {
    // Defined in <bits/mman-linux.h>
    #[derive(Default)]
    pub struct MmapFlags: i32 {
        // Sharing types (must choose one and only one of these).
        /// Share changes.
        const MAP_SHARED = 0x01;
        /// Changes are private.
        const MAP_PRIVATE = 0x02;
        /// Share changes and validate
        const MAP_SHARED_VALIDATE = 0x03;
        const MAP_TYPE_MASK = 0x03;

        // Other flags
        /// Interpret addr exactly.
        const MAP_FIXED = 0x10;
        /// Don't use a file.
        const MAP_ANONYMOUS = 0x20;
        /// Don't check for reservations.
        const MAP_NORESERVE = 0x04000;
    }
}

bitflags! {
    // Defined in <bits/mman-linux.h>
    // NOTE: Zero bit flag is discouraged. See https://docs.rs/bitflags/latest/bitflags/#zero-bit-flags
    pub struct MmapProt: i32 {
        /// Page can be read.
        const PROT_READ = 0x1;
        /// Page can be written.
        const PROT_WRITE = 0x2;
        /// Page can be executed.
        const PROT_EXEC = 0x4;
    }
}

bitflags! {
    /// The flags bit-mask argument may be 0, or include the following flags
    pub struct MremapFlags: i32 {
        ///  By default, if there is not sufficient space to expand a mapping at its current location, then mremap()
        ///  fails.   If  this flag is specified, then the kernel is permitted to relocate the mapping to a new vir‐
        ///  tual address, if necessary.  If the mapping is relocated, then absolute pointers into the  old  mapping
        ///  location become invalid (offsets relative to the starting address of the mapping should be employed).
        const MAYMOVE    = 1 << 0;
        /// This  flag  serves a similar purpose to the MAP_FIXED flag of mmap(2).  If this flag is specified, then
        /// mremap() accepts a fifth argument, void *new_address, which specifies a page-aligned address  to  which
        /// the  mapping  must  be  moved.   Any previous mapping at the address range specified by new_address and
        /// new_size is unmapped.
        /// 
        /// If MREMAP_FIXED is specified, then MREMAP_MAYMOVE must also be specified.
        const FIXED      = 1 << 1;
        /// This flag, which must be used in conjunction with MREMAP_MAYMOVE, remaps a mapping to a new address but
        /// does not unmap the mapping at old_address.
        /// 
        /// The MREMAP_DONTUNMAP flag can be used only with private anonymous  mappings  (see  the  description  of
        /// MAP_PRIVATE and MAP_ANONYMOUS in mmap(2)).
        /// 
        /// After  completion,  any access to the range specified by old_address and old_size will result in a page
        /// fault.  The page fault will be handled by a userfaultfd(2) handler if the address is in a range  previ‐
        /// ously registered with userfaultfd(2).  Otherwise, the kernel allocates a zero-filled page to handle the
        /// fault.
        /// 
        /// The  MREMAP_DONTUNMAP  flag  may  be used to atomically move a mapping while leaving the source mapped.
        /// See NOTES for some possible applications of MREMAP_DONTUNMAP.
        const DONTUNMAP  = 1 << 2;
    }
}

pub fn mmap(addr: usize, len: usize, prot: MmapProt, flags: MmapFlags, fd: usize, offset: usize) -> isize {
    sys_mmap(addr, len, prot.bits, flags.bits, fd, offset)
}

pub fn mremap(old_addr: usize, old_size: usize, new_size: usize, flags: MremapFlags, new_addr:usize) -> isize {
    sys_mremap(old_addr, old_size, new_size, flags.bits, new_addr)
}

pub fn shutdown() -> isize {
    sys_shutdown(0, 0, 0, 0)
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
/// TimeVal struct for syscall, TimeVal stans for low-precision time value
pub struct TimeVal {
    /// seconds
    pub sec: usize,
    /// microseconds
    pub usec: usize,
}