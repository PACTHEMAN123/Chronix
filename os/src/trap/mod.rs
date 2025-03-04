//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].
mod context;

use crate::async_utils::yield_now;
use crate::config::TRAP_CONTEXT;
use crate::mm::{PageFaultAccessType, VirtAddr, VmSpacePageFaultExt};
use crate::syscall::syscall;
use crate::task::{
     current_user_token, exit_current_and_run_next, suspend_current_and_run_next, current_task,
};
use crate::task::processor::current_trap_cx;
use crate::timer::set_next_trigger;
use core::arch::{asm, global_asm};
use crate::arch::riscv64::interrupts::disable_interrupt;
use alloc::task;
use log::{info, warn};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec, sepc,
};

global_asm!(include_str!("trap.S"));
/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
}

/// set the kernel trap entry
pub fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}
/// enable timer interrupt in sie CSR
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
/// handle an interrupt, exception, or system call from user space
pub async fn trap_handler()  {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    let sepc = sepc::read();
    let cause = scause.cause(); 
    /*info!(
        "[trap_handler] scause: {:?}, stval: {:#x}, sepc: {:#x}",
        cause, stval, sepc
    ); */
    
    //unsafe { enable_interrupt() };
   
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]).await;
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            log::debug!(
                "[trap_handler] encounter page fault, addr {stval:#x}, instruction {sepc:#x} scause {cause:?}",
            );

            let access_type = match scause.cause() {
                Trap::Exception(Exception::InstructionPageFault) => PageFaultAccessType::EXECUTE,
                Trap::Exception(Exception::LoadPageFault) => PageFaultAccessType::READ,
                Trap::Exception(Exception::StorePageFault) => PageFaultAccessType::WRITE,
                _ => unreachable!(),
            };

           match current_task() {
                None => {},
                Some(task) => {
                    let res = task.inner_exclusive_access().vm_space.handle_page_fault(VirtAddr::from(stval), access_type);
                    match res {
                        Some(_) => {},
                        None => {
                            // todo: don't panic, kill the task
                            panic!(
                                "[trap_handler] cannot handle page fault, addr {stval:#x}, instruction {sepc:#x} scause {cause:?}",
                            );
                        }
                    }
                }
            };

        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::LoadFault) => {
            println!(
                "[trap_handler] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[trap_handler] IllegalInstruction in application, kernel killed it.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            //info!("interrupt: supervisor timer");
            set_next_trigger();
            yield_now().await;
        }
        _ => {
            panic!(
                "[trap_handler] Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    //println!("before trap_return");
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() {
    unsafe{
        disable_interrupt();
    }
    //info!("trap return, user sp {:#x}, kernel sp {:#x}", current_trap_cx().x[2], current_trap_cx().kernel_sp);
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    //let user_satp = current_user_token();
    extern "C" {
        fn __restore();
    }
    unsafe {
        asm!(
            "call __restore",    
            in("a0") trap_cx_ptr,        
        );
    }
}

#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn trap_from_kernel() -> ! {
    use riscv::register::sepc;
    println!("stval = {:#x}, sepc = {:#x}", stval::read(), sepc::read());
    panic!("a trap {:?} from kernel!", scause::read().cause());
}

pub use context::TrapContext;
