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

use alloc::sync::Arc;
use hal::constant::{Constant, ConstantsHal};
use hal::instruction::{self, Instruction, InstructionHal};
use hal::pagetable::PageTableHal;
use hal::println;
use hal::trap::{set_kernel_trap_entry, set_user_trap_entry, TrapContext, TrapContextHal, TrapType, TrapTypeHal};
use crate::mm::vm::{KernVmSpaceHal, PageFaultAccessType, UserVmSpaceHal};
use crate::mm::KVMSPACE;
use hal::addr::VirtAddr;

use crate::utils::async_utils::yield_now;
use crate::executor;
use crate::processor::context::SumGuard;
use crate::syscall::{syscall, SysError};
use crate::task::task::TaskControlBlock;
use crate::task::{
     current_user_token, exit_current_and_run_next, suspend_current_and_run_next, current_task,
};
use crate::processor::processor::{current_processor, current_trap_cx};
use crate::timer::set_next_trigger;
use core::arch::{asm, global_asm};
use alloc::task;
use log::{info, warn};
use core::sync::atomic::Ordering;

hal::define_user_trap_handler!(user_trap_handler);

/// handle an interrupt, exception, or system call from user space
/// return true if it is syscall and has been interrupted
pub async fn user_trap_handler() -> bool {
    set_kernel_trap_entry();
    let trap_type = TrapType::get();
    unsafe { Instruction::enable_interrupt() };
    match trap_type {
        TrapType::Breakpoint => {
            let task = current_task().unwrap();
            log::warn!(
                "[user_trap_handler] task {} break point",
                task.tid()
            );
            exit_current_and_run_next(-1);
        }
        TrapType::Syscall => {
            let _sum = SumGuard::new();
            let cx = unsafe {
                &mut *(Constant::USER_TRAP_CONTEXT_BOTTOM as *mut TrapContext)
            };
            // jump to next instruction anyway
            *cx.sepc() += 4;
            // get system call return value
            let result = syscall(
                cx.syscall_id(), 
                [
                    cx.syscall_arg_nth(0), 
                    cx.syscall_arg_nth(1), 
                    cx.syscall_arg_nth(2), 
                    cx.syscall_arg_nth(3), 
                    cx.syscall_arg_nth(4), 
                    cx.syscall_arg_nth(5)
                ]
            ).await;
            // save last user arg0 to restore for possible SA_RESTART flag in signal
            cx.save_last_user_arg0();
            // cx is changed during sys_exec, so we have to call it again
            cx.save_to(0, cx.ret_nth(0));
            cx.set_ret_nth(0, result as usize);
            // report that the syscall is interrupt
            if result == SysError::EINTR as isize {
                return true;
            }
        }
        TrapType::StorePageFault(stval)
        | TrapType::InstructionPageFault(stval)
        | TrapType::LoadPageFault(stval) => {
            log::debug!(
                "[user_trap_handler] encounter page fault, addr {stval:#x}",
            );

            let access_type = match trap_type {
                TrapType::StorePageFault(_) => PageFaultAccessType::WRITE,
                TrapType::LoadPageFault(_) => PageFaultAccessType::READ,
                TrapType::InstructionPageFault(_) => PageFaultAccessType::EXECUTE,
                _ => unreachable!(),
            };

            match current_task() {
                None => {},
                Some(task) => {
                    let res = task.with_mut_vm_space(|vm_space| vm_space.handle_page_fault(VirtAddr::from(stval), access_type));
                    match res {
                        Ok(()) => {}
                        Err(()) => {
                            log::warn!(
                                "[user_trap_handler] cannot handle page fault, addr {stval:#x} access_type: {access_type:?}",
                            );
                            exit_current_and_run_next(-2);
                        }
                    }
                }
            };
        }
        TrapType::IllegalInstruction(_) => {
            println!("[trap_handler] IllegalInstruction in application, kernel killed it.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        TrapType::Timer => {
            // println!("timer interp");
            crate::timer::timer::TIMER_MANAGER.check();
            #[cfg(feature = "smp")]
            crate::processor::processor::current_processor().update_load_avg();
            set_next_trigger();
            yield_now().await;
        }
        TrapType::Processed => {}
        _ => {
            panic!(
                "[trap_handler] Unsupported trap!"
            );
        }
    }
    false
    // println!("before trap_return");
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return(task: &Arc<TaskControlBlock>, is_intr: bool) {
    unsafe {
        Instruction::disable_interrupt();  
    }
    set_user_trap_entry();
    
    task.time_recorder().record_trap_return();

    let trap_cx_ptr = Constant::USER_TRAP_CONTEXT_BOTTOM;

    // handler the signal before return
    task.check_and_handle(is_intr);

    // restore float pointer and set status
    task.get_trap_cx().fx_restore();
    
    Instruction::set_float_status_clean();
    // restore
    hal::trap::restore(trap_cx_ptr);

    task.get_trap_cx().mark_fx_save();

    // set up time recorder for trap
    task.time_recorder().record_trap();
    // info!("[in record trap] task id: {}kernel_time:{:?}",task.tid(),task.time_recorder().kernel_time());
}

hal::define_kernel_trap_handler!(kernel_trap_handler);

/// Kernel trap handler
fn kernel_trap_handler() {
    let (trap_type, epc) = TrapType::get_debug();
    match trap_type {
        TrapType::StorePageFault(stval)
        | TrapType::LoadPageFault(stval)
        | TrapType::InstructionPageFault(stval) => {
            log::debug!(
                "[kernel_trap_handler] encounter page fault, addr {stval:#x}",
            );

            let access_type = match trap_type {
                TrapType::StorePageFault(_) => PageFaultAccessType::WRITE,
                TrapType::LoadPageFault(_) => PageFaultAccessType::READ,
                TrapType::InstructionPageFault(_) => PageFaultAccessType::EXECUTE,
                _ => unreachable!(),
            };

            if KVMSPACE.lock().handle_page_fault(VirtAddr::from(stval), access_type).is_err() {
                match current_task() {
                    None => {},
                    Some(task) => {
                        let res = task.with_mut_vm_space(|vm_space|vm_space.handle_page_fault(VirtAddr::from(stval), access_type));
                        match res {
                            Ok(()) => {},
                            Err(()) => {
                                // todo: don't panic, kill the task
                                panic!(
                                    "[kernel_trap_handler] cannot handle page fault, addr {stval:#x}, access type: {access_type:?}, epc: {epc:#x}",
                                );
                            }
                        }
                    }
                };
            }
        }
        TrapType::Timer => {
            //info!("interrupt: supervisor timer");
            crate::timer::timer::TIMER_MANAGER.check();
            set_next_trigger();
        }
        TrapType::Processed => {}
        _ => {
            // error!("other exception!!");
            panic!(
                "a unsupported trap {:?} from kernel!",
                trap_type,
            );
        }
    }
}