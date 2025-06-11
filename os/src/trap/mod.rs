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
use downcast_rs::Downcast;
use hal::constant::{Constant, ConstantsHal};
use hal::instruction::{self, Instruction, InstructionHal};
use hal::pagetable::PageTableHal;
use hal::println;
use hal::trap::{set_kernel_trap_entry, set_user_trap_entry, TrapContext, TrapContextHal, TrapType, TrapTypeHal};
use hal::util::backtrace;
use crate::mm::vm::{KernVmSpaceHal, PageFaultAccessType, UserVmSpaceHal};
use crate::mm::KVMSPACE;
use crate::signal::{SigInfo, SIGILL, SIGKILL, SIGSEGV, SIGTRAP};
use crate::utils::timer::TimerGuard;
use hal::addr::VirtAddr;

use crate::utils::async_utils::yield_now;
use crate::executor;
use crate::processor::context::SumGuard;
use crate::syscall::{syscall, SysError};
use crate::task::task::TaskControlBlock;
use crate::task::{
     current_user_token, current_task,
};
use crate::processor::processor::{current_processor, current_trap_cx};
use crate::timer::set_next_trigger;
use core::arch::{asm, global_asm};
use alloc::{format, task};
use log::{info, warn};
use core::sync::atomic::Ordering;

hal::define_user_trap_handler!(user_trap_handler);

/// handle an interrupt, exception, or system call from user space
/// return true if it is syscall and has been interrupted
pub async fn user_trap_handler() -> bool {
    set_kernel_trap_entry();
    let (trap_type, epc) = TrapType::get_debug();
    unsafe { Instruction::enable_interrupt() };
    match trap_type {
        TrapType::Breakpoint => {
            let task = current_task().unwrap();
            log::warn!(
                "[user_trap_handler] task {} break point",
                task.tid()
            );
            let task = current_task().unwrap().clone();
            // task.set_stopped();
            task.recv_sigs(SigInfo { si_signo: SIGTRAP, si_code: SigInfo::KERNEL, si_pid: None });
        }
        TrapType::Syscall => {
            let _sum = SumGuard::new();
            let cx = current_task().unwrap().get_trap_cx();
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
            // // cx is changed during sys_exec, so we have to call it again
            // cx.save_to(0, cx.ret_nth(0));
            // report that the syscall is interrupt
            cx.set_ret_nth(0, result as usize);
            if result == -(SysError::EINTR as isize) {
                log::warn!("[user_trap_handler] task {} syscall is interrupted", cx.syscall_id());
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

            let task = current_task().unwrap();
            let res = task.with_mut_vm_space(|vm_space| vm_space.handle_page_fault(VirtAddr::from(stval), access_type));
            match res {
                Ok(()) => {}
                Err(()) => {
                    log::warn!(
                        "[user_trap_handler] task pid {}, tid {}, cannot handle page fault, addr {stval:#x} access_type: {access_type:?} epc: {epc:#x}",
                        task.pid(), task.tid()
                    );
                    task.recv_sigs(SigInfo { si_signo: SIGSEGV, si_code: SigInfo::KERNEL, si_pid: None });
                }
            }
        }
        TrapType::IllegalInstruction(_) => {
            println!("[trap_handler] IllegalInstruction in application, kernel killed it.");
            // illegal instruction exit code
            let task = current_task().unwrap();
            task.recv_sigs(SigInfo { si_signo: SIGILL, si_code: SigInfo::KERNEL, si_pid: None });
        }
        TrapType::Timer => {
            crate::timer::timer::TIMER_MANAGER.check();
            #[cfg(feature = "smp")]
            crate::processor::processor::current_processor().update_load_avg();
            set_next_trigger();
            yield_now().await;
        }
        TrapType::ExternalInterrupt => {
            let manager = crate::devices::DEVICE_MANAGER.lock();
            manager.handle_irq();
        }
        TrapType::Processed => {}
        trap => {
            panic!(
                "[trap_handler] Unsupported trap! {:?}", trap
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
pub fn trap_return(task: &Arc<TaskControlBlock>, _is_intr: bool) {
    unsafe {
        Instruction::disable_interrupt();  
    }
    set_user_trap_entry();
    
    task.time_recorder().record_trap_return();

    let trap_cx = task.get_trap_cx();

    // handler the signal before return
    // task.check_and_handle(is_intr);

    // restore float pointer and set status
    trap_cx.fx_restore();
    
    Instruction::set_float_status_clean();
    // restore
    hal::trap::restore(trap_cx);
    
    trap_cx.mark_fx_save();

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
            // warn: page fault from kernel is dangerous
            log::warn!(
                "[kernel_trap_handler] encounter page fault, addr {stval:#x} epc {epc:#x}",
            );
            // backtrace();

            let access_type = match trap_type {
                TrapType::StorePageFault(_) => PageFaultAccessType::WRITE,
                TrapType::LoadPageFault(_) => PageFaultAccessType::READ,
                TrapType::InstructionPageFault(_) => PageFaultAccessType::EXECUTE,
                _ => unreachable!(),
            };

            match current_task() {
                None => {
                    panic!(
                        "[kernel_trap_handler] cannot handle page fault, addr {stval:#x}, access type: {access_type:?}, epc: {epc:#x}"
                    );
                },
                Some(task) => {
                    let res = task.with_mut_vm_space(|vm_space|vm_space.handle_page_fault(VirtAddr::from(stval), access_type));
                    match res {
                        Ok(()) => {},
                        Err(()) => {
                            panic!(
                                "[kernel_trap_handler] cannot handle page fault, task {}, addr {stval:#x}, access type: {access_type:?}, epc: {epc:#x}",
                                task.tid()
                            );
                        }
                    }
                }
            };
        }
        TrapType::Timer => {
            // println!("interrupt: supervisor timer");
            crate::timer::timer::TIMER_MANAGER.check();
            set_next_trigger();
        }
        TrapType::ExternalInterrupt => {
            let manager = crate::devices::DEVICE_MANAGER.lock();
            manager.handle_irq();
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