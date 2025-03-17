use core::arch::global_asm;
use hal::{addr::{VirtAddr, VirtAddrHal, VirtPageNumHal}, constant::{Constant, ConstantsHal}, trap::set_kernel_trap_entry};
use riscv::register::{
    scause::Scause, 
    mtvec::TrapMode,
    stvec,
    sstatus,
};
use log::*;

use crate::{processor::context::SumGuard, sync::mutex::SieGuard};

global_asm!(include_str!("check.S"));

#[derive(Clone, Copy)]
#[repr(C)]
struct CheckResult {
    is_err: usize,
    scause: usize,
}

unsafe extern "C" {
    unsafe fn __try_access_user_error_trap();
    unsafe fn __try_read_user_u8(user_addr: usize) -> CheckResult;
    unsafe fn __try_write_user_u8(user_addr: usize) -> CheckResult;
    unsafe fn __trap_from_user();
    unsafe fn __trap_from_kernel();
}

/// UserCheck struct for user pointer check
pub struct UserCheck {
    _sum_guard: SumGuard,
    _sie_guard: SieGuard,
}

impl UserCheck {
    /// create a new user check
    pub fn new() -> Self {
        let ret = Self {
            _sum_guard: SumGuard::new(),
            _sie_guard: SieGuard::new(),
        };
        unsafe {
            stvec::write(__try_access_user_error_trap as usize, TrapMode::Direct);
        }
        ret
    }

    /// check if the given user address is readable or not
    pub fn check_read_slice(&self, buf: *const u8, len: usize) {
        info!("into user check and read");
        let buf_start: VirtAddr = VirtAddr::from(buf as usize).floor().start_addr();
        let buf_end: VirtAddr = VirtAddr::from(buf as usize + len).ceil().start_addr();
        
        assert!(buf_start < buf_end);

        let mut va = buf_start;
        while va < buf_end {
            if let Some(scause) = self.check_read_u8(va.0) {
                // todo: into the page fault handler
                panic!("user read page fault: {:?}, va: {:?}", scause, va.0);
            }
            va.0 += Constant::PAGE_SIZE;
        }
        info!("exit user check and read");
        // if nothing panic, indicate readable
    }

    /// check if the given user address is writable or not
    pub fn check_write_slice(&self, buf: *mut u8, len: usize) {
        info!("into user check and write");
        info!("buf_start: {:#x}, buf_end: {:#x}", buf as usize, buf as usize + len);
        let buf_start: VirtAddr = VirtAddr::from(buf as usize).floor().start_addr();
        let buf_end: VirtAddr = VirtAddr::from(buf as usize + len).ceil().start_addr();
        
        
        assert!(buf_start < buf_end);

        let mut va = buf_start;
        while va < buf_end {
            if let Some(scause) = self.check_write_u8(va.0) {
                // todo: into the page fault handler
                panic!("user write page fault: {:?}, va: {:#x}", scause, va.0);
            }
            va.0 += Constant::PAGE_SIZE;
        }
        info!("exit user check and write");
        // if nothing panic, indicate writable
    }

    /// check writable of a single u8
    fn check_write_u8(&self, addr: usize) -> Option<usize> {
        let ret = unsafe { __try_write_user_u8(addr) };
        match ret.is_err {
            0 => None,
            _ => Some(ret.scause),
        }
    }

    /// check readable of a single u8
    fn check_read_u8(&self, addr: usize) -> Option<usize> {
        let ret = unsafe { __try_read_user_u8(addr) };
        match ret.is_err {
            0 => None,
            _ => Some(ret.scause),
        }
    }
        
}

impl Drop for UserCheck {
    fn drop(&mut self) {
        set_kernel_trap_entry();
    }
}
