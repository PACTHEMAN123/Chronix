//! addition struct and some useful helper function for task

use alloc::{string::String, vec::Vec};
use hal::{addr::VirtAddr, println};

use crate::{config::PAGE_SIZE, mm::{PageTable, UserVmSpace}, processor::context::SumGuard};
use crate::mm::vm::{self, PageFaultAccessType, UserVmSpaceHal};

/// end of vector
pub const AT_NULL: usize = 0;
/// entry should be ignored
#[allow(unused)]
pub const AT_IGNORE: usize = 1;
/// file descriptor of program
#[allow(unused)]
pub const AT_EXECFD: usize = 2;
/// program headers for program
pub const AT_PHDR: usize = 3;
/// size of program header entry
pub const AT_PHENT: usize = 4;
/// number of program headers
pub const AT_PHNUM: usize = 5;
/// system page size
pub const AT_PAGESZ: usize = 6;
/// base address of interpreter
pub const AT_BASE: usize = 7;
/// flags
pub const AT_FLAGS: usize = 8;
/// entry point of program
pub const AT_ENTRY: usize = 9;
/// program is not ELF
#[allow(unused)]
pub const AT_NOTELF: usize = 10;
/// real uid
pub const AT_UID: usize = 11;
/// effective uid
pub const AT_EUID: usize = 12;
/// real gid
pub const AT_GID: usize = 13;
/// effective gid
pub const AT_EGID: usize = 14;
/// string identifying CPU for optimizations
pub const AT_PLATFORM: usize = 15;
/// arch dependent hints at CPU capabilities
pub const AT_HWCAP: usize = 16;
/// frequency at which times() increments
pub const AT_CLKTCK: usize = 17;

/// AT_* values 18 through 22 are reserved

/// secure mode boolean
pub const AT_SECURE: usize = 23;
/// string identifying real platform, may differ from AT_PLATFORM
#[allow(unused)]
pub const AT_BASE_PLATFORM: usize = 24;
/// address of 16 random bytes
// NOTE: libc may use these 16 bytes as stack check guard, therefore, the
// address must be valid
pub const AT_RANDOM: usize = 25;
/// extension of AT_HWCAP
#[allow(unused)]
pub const AT_HWCAP2: usize = 26;
/// filename of program
pub const AT_EXECFN: usize = 31;
/// entry point to the system call function in the vDSO
#[allow(unused)]
pub const AT_SYSINFO: usize = 32;
/// address of a page containing the vDSO
#[allow(unused)]
pub const AT_SYSINFO_EHDR: usize = 33;

/// Auxiliary header
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AuxHeader {
    /// Type
    pub aux_type: usize,
    /// Value
    pub value: usize,
}

impl AuxHeader {
    /// new a AuxHeader
    pub fn new(aux_type: usize, value: usize) -> Self {
        Self { aux_type, value }
    }
}

/// create a new auxv
/// need more initialize
pub fn generate_early_auxv(
    ph_entry_size: usize,
    ph_count: usize,
    entry_point: usize,
) -> Vec<AuxHeader> {
    let mut auxv = Vec::with_capacity(32);
    macro_rules! push {
        ($x1:expr, $x2:expr) => {
            auxv.push(AuxHeader::new($x1, $x2));
        };
    }
    push!(AT_PHENT, ph_entry_size);
    push!(AT_PHNUM, ph_count);
    push!(AT_PAGESZ, PAGE_SIZE);
    push!(AT_FLAGS, 0);
    push!(AT_ENTRY, entry_point);
    push!(AT_UID, 0);
    push!(AT_EUID, 0);
    push!(AT_GID, 0);
    push!(AT_EGID, 0);
    push!(AT_PLATFORM, 0);
    push!(AT_HWCAP, 0);
    push!(AT_CLKTCK, 100);
    push!(AT_SECURE, 0);
    auxv
}


/// helper function to push argc, argv, envp, auxv
/// and other infomation into user stack
/// when execve init the user space
/// NOTICE: before calling this,
/// hart page table should already change.
pub fn user_stack_init(
    vm_space: &mut UserVmSpace,
    sp: usize, 
    argv: Vec<String>, 
    envp: Vec<String>, 
    auxv: Vec<AuxHeader>,
) -> (usize, usize, usize, usize) {
    let _sum_guard = SumGuard::new();
    let platfrom = "RISC-V64";
    let rand_bytes = "Chronix Is Here"; // 15 + 1 char for 16 bytes
    let rand_size = 0usize;

    // calculate the total size from stack buttom to top
    let mut new_sp = sp;
    // args string end with '/0'
    new_sp -= argv.iter().map(|s|s.as_bytes().len() + 1).sum::<usize>();
    let program_name_ptr = new_sp;
    // env strings end with '/0'
    new_sp -= envp.iter().map(|s|s.as_bytes().len() + 1).sum::<usize>();
    // random aligned (use 0 aligned here)
    new_sp -= rand_size;
    // platfrom string end with '/0'
    new_sp -= platfrom.as_bytes().len() + 1;
    // random 16 bytes
    new_sp -= rand_bytes.as_bytes().len() + 1;
    // aligned to 16
    new_sp = (new_sp - 1) & !0xf;
    // auxv vec and a null auxv
    new_sp -= (auxv.len() + 1) * core::mem::size_of::<AuxHeader>();
    // envp
    new_sp -= (envp.len() + 1) * core::mem::size_of::<usize>();
    // argv
    new_sp -= (envp.len() + 1) * core::mem::size_of::<usize>();
    // argc
    new_sp -= core::mem::size_of::<usize>();

    // we need to use page fault to activate the needed space
    let frames_num = ((sp - new_sp) + PAGE_SIZE - 1) / PAGE_SIZE;
    for i in 1..frames_num+1 {
        let _ = vm_space.handle_page_fault(VirtAddr::from(sp - PAGE_SIZE * i), PageFaultAccessType::WRITE);
    }

    // push the data into stack in the order mention above
    let mut new_sp = sp;
    // env arg strings
    let env_ptrs: Vec<usize> = envp.iter().rev().map(|s| push_str(&mut new_sp, s)).collect();
    let arg_ptrs: Vec<usize> = argv.iter().rev().map(|s| push_str(&mut new_sp, s)).collect();
    // platfrom, rand bytes, align bytes
    new_sp -= rand_size;
    push_str(&mut new_sp, platfrom);
    push_str(&mut new_sp, rand_bytes);
    align16(&mut new_sp);
    // aux
    push_aux(&mut new_sp, &AuxHeader::new(AT_NULL, 0));
    push_aux(&mut new_sp, &AuxHeader::new(AT_EXECFN, program_name_ptr));
    for aux in auxv.into_iter().rev() {
        push_aux(&mut new_sp, &aux);
    }
    // env
    push_usize(&mut new_sp, 0);
    env_ptrs.iter().for_each(|ptr| {push_usize(&mut new_sp, *ptr);});
    let env_ptr = new_sp;
    // arg
    push_usize(&mut new_sp, 0);
    arg_ptrs.iter().for_each(|ptr| {push_usize(&mut new_sp, *ptr);});
    let arg_ptr = new_sp;
    // argc
    let argc = argv.len();
    push_usize(&mut new_sp, argc);

    (new_sp, argc, arg_ptr, env_ptr) 
}

/// push rust string appending a '/0' into user stack
pub fn push_str(sp: &mut usize, s: &str) -> usize {
    let len = s.len();
    *sp -= len + 1; // +1 for NUL ('\0')
    unsafe {
        for (i, c) in s.bytes().enumerate() {
            log::trace!(
                "push_str: {:x} ({:x}) <- {:?}",
                *sp + i,
                i,
                core::str::from_utf8_unchecked(&[c])
            );
            *((*sp as *mut u8).add(i)) = c;
        }
        *(*sp as *mut u8).add(len) = 0u8;
    }
    *sp
}

/// push aux header into user stack
pub fn push_aux(sp: &mut usize, elm: &AuxHeader) {
    *sp -= core::mem::size_of::<AuxHeader>();
    unsafe {
        core::ptr::write(*sp as *mut AuxHeader, *elm);
    }
}

/// push pointer into user stack
pub fn push_usize(sp: &mut usize, ptr: usize) {
    *sp -= core::mem::size_of::<usize>();
    unsafe {
        core::ptr::write(*sp as *mut usize, ptr);
    }
}

/// align the addr to 16
pub fn align16(sp: &mut usize) {
    *sp = (*sp - 1) & !0xf;
}