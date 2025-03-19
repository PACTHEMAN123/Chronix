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
use hal::instruction::{Instruction, InstructionHal};
use hal::println;
use hal::trap::{set_kernel_trap_entry, set_user_trap_entry, TrapContext, TrapContextHal, TrapType};
use hal::vm::UserVmSpaceHal;
use hal::{addr::VirtAddr, vm::PageFaultAccessType};

use crate::utils::async_utils::yield_now;
use crate::executor;
use crate::processor::context::SumGuard;
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
use core::sync::atomic::Ordering;

hal::define_user_trap_handler!(user_trap_handler);

/// handle an interrupt, exception, or system call from user space
async fn user_trap_handler(trap_type: TrapType)  {
    set_kernel_trap_entry();
    unsafe { Instruction::enable_interrupt() };
    match trap_type{
        TrapType::Syscall => {
            /*
            let cur_processor = current_processor();
            let cx = current_trap_cx(cur_processor);
            */
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
            // cx is changed during sys_exec, so we have to call it again
            cx.save_to(0, cx.ret_nth(0));
            cx.set_ret_nth(0, result as usize);
        }
        TrapType::StorePageFault(stval)
        | TrapType::InstructionPageFault(stval)
        | TrapType::LoadPageFault(stval) => {
            log::debug!(
                "[trap_handler] encounter page fault, addr {stval:#x}",
            );

            let access_type = match trap_type {
                TrapType::StorePageFault(_) => PageFaultAccessType::READ,
                TrapType::LoadPageFault(_) => PageFaultAccessType::WRITE,
                TrapType::InstructionPageFault(_) => PageFaultAccessType::EXECUTE,
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
                                "[user_trap_handler] cannot handle page fault, addr {stval:#x}",
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
            crate::timer::timer::TIMER_MANAGER.check();
            set_next_trigger();
            yield_now().await;
        }
        _ => {
            /*panic!(
                "[trap_handler] Unsupported trap!"
            );
            */
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

    set_user_trap_entry();
    task.time_recorder().record_trap_return();
    //info!("hart_id:{},task time record: user_time:{:?},kernel_time:{:?}",current_processor().id(),task.time_recorder().user_time(),task.time_recorder().kernel_time());
    let trap_cx_ptr = Constant::USER_TRAP_CONTEXT_BOTTOM;

    // handler the signal before return
    check_signal_for_current_task();
    hal::trap::restore(trap_cx_ptr);
    task.time_recorder().record_trap();
    //info!("hart_id:{},task time record: user_time:{:?},kernel_time:{:?}",current_processor().id(),task.time_recorder().user_time(),task.time_recorder().kernel_time());
}

hal::define_kernel_trap_handler!(kernel_trap_handler);

/// Kernel trap handler
fn kernel_trap_handler(trap_type: TrapType) {
    match trap_type {
        TrapType::StorePageFault(stval)
        | TrapType::LoadPageFault(stval)
        | TrapType::InstructionPageFault(stval) => {
            log::debug!(
                "[trap_handler] encounter page fault, addr {stval:#x}",
            );

            let access_type = match trap_type {
                TrapType::StorePageFault(_) => PageFaultAccessType::READ,
                TrapType::LoadPageFault(_) => PageFaultAccessType::WRITE,
                TrapType::InstructionPageFault(_) => PageFaultAccessType::EXECUTE,
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
                                "[kernel_trap_handler] cannot handle page fault, addr {stval:#x}",
                            );
                        }
                    }
                }
            };

        }
        TrapType::Timer => {
            //info!("interrupt: supervisor timer");
            crate::timer::timer::TIMER_MANAGER.check();
            set_next_trigger();
        }
        _ => {
            // error!("other exception!!");
            panic!(
                "a unsupported trap {:?} from kernel!",
                trap_type
            );
        }
    }
}