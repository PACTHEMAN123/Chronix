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

use alloc::sync::Arc;
use hal::constant::{Constant, ConstantsHal};
use hal::instruction::{Instruction, InstructionHal};
use hal::println;
use hal::vm::UserVmSpaceHal;
use hal::{addr::VirtAddr, vm::PageFaultAccessType};

use crate::utils::async_utils::yield_now;
use crate::executor;
use crate::signal::check_signal_for_current_task;
use crate::syscall::syscall;
use crate::task::task::TaskControlBlock;
use crate::task::{
     current_user_token, exit_current_and_run_next, suspend_current_and_run_next, current_task,
};
use crate::processor::processor::{current_processor, current_trap_cx};
use crate::timer::set_next_trigger;
use core::arch::{asm, global_asm};
use alloc::task;
use log::{info, warn};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec, sepc,
};
use core::sync::atomic::Ordering;

global_asm!(include_str!("trap.S"));
/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
}

/// set the kernel trap entry
pub fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(__trap_from_kernel as usize, TrapMode::Direct);
    }
}
extern "C" {
    fn __alltraps();
    fn __trap_from_kernel();
}
fn set_user_trap_entry() {
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
    
    unsafe { Instruction::enable_interrupt() };
   
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let current_processor = current_processor();
            // jump to next instruction anyway
            let cx = current_trap_cx(current_processor);
            cx.sepc += 4;
            // get system call return value
            let result = syscall(
                cx.x[17], 
                [cx.x[10], cx.x[11], cx.x[12], cx.x[13], cx.x[14], cx.x[15]]
            ).await;
            // cx is changed during sys_exec, so we have to call it again
            cx.save_last_a0();
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
                    let res = task.with_mut_vm_space(|vm_space| vm_space.handle_page_fault(VirtAddr::from(stval), access_type));
                    match res {
                        Ok(()) => {},
                        Err(()) => {
                            // todo: don't panic, kill the task
                            log::warn!(
                                "[trap_handler] cannot handle page fault, addr {stval:#x}, instruction {sepc:#x} scause {cause:?}",
                            );
                            exit_current_and_run_next(-4);
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
                current_trap_cx(current_processor()).sepc,
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
pub fn trap_return(task: &Arc<TaskControlBlock>) {
    unsafe{
        Instruction::disable_interrupt();
    }
    //info!("trap return, user sp {:#x}, kernel sp {:#x}", current_trap_cx().x[2], current_trap_cx().kernel_sp);
    set_user_trap_entry();
    task.time_recorder().record_trap_return();
    //info!("hart_id:{},task time record: user_time:{:?},kernel_time:{:?}",current_processor().id(),task.time_recorder().user_time(),task.time_recorder().kernel_time());
    let trap_cx_ptr = Constant::USER_TRAP_CONTEXT_BOTTOM;
    //let user_satp = current_user_token();
    extern "C" {
        fn __restore();
    }
    // handler the signal before return
    check_signal_for_current_task();
    unsafe {
        asm!(
            "call __restore",    
            in("a0") trap_cx_ptr,        
        );
    }
    task.time_recorder().record_trap();
    //info!("hart_id:{},task time record: user_time:{:?},kernel_time:{:?}",current_processor().id(),task.time_recorder().user_time(),task.time_recorder().kernel_time());
}

pub use context::TrapContext;

#[no_mangle]
/// Kernel trap handler
pub fn kernel_trap_handler() {
    let scause = scause::read();
    let sepc = sepc::read();
    let cause = scause.cause();
    let stval = stval::read();
    match scause.cause() {
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
                    let res = task.with_mut_vm_space(|vm_space|vm_space.handle_page_fault(VirtAddr::from(stval), access_type));
                    match res {
                        Ok(()) => {},
                        Err(()) => {
                            // todo: don't panic, kill the task
                            panic!(
                                "[trap_handler] cannot handle page fault, addr {stval:#x}, instruction {sepc:#x} scause {cause:?}",
                            );
                        }
                    }
                }
            };

        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            //info!("interrupt: supervisor timer");
            set_next_trigger();
        }
        _ => {
            // error!("other exception!!");
            info!(
                "[kernel] {:?}(scause:{}) in application, bad addr = {:#x}, bad instruction = {:#x}, kernel panicked!!",
                scause::read().cause(),
                scause::read().bits(),
                stval::read(),
                sepc::read(),
            );
            panic!(
                "a trap {:?} from kernel! stval {:#x}",
                scause::read().cause(),
                stval::read()
            );
        }
    }
}