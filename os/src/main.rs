//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`task`]: Task management
//! - [`syscall`]: System call handling and implementation
//! - [`mm`]: Address map using SV39
//! - [`sync`]: Wrap a static data structure inside it so that we are able to access it without any `unsafe`.
//! - [`fs`]: Separate user from file system with some structures
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`task::run_tasks()`] and for the first time go to
//! userspace.

#![feature(negative_impls)]
// #![deny(missing_docs)]
// #![deny(warnings)]
#![allow(unused_imports)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(step_trait)]
#![feature(new_range_api)]
#![feature(naked_functions)]
#![feature(allocator_api)]
#![feature(btreemap_alloc)]
#![feature(arbitrary_self_types)]
#![feature(new_zeroed_alloc)]
extern crate alloc;

#[macro_use]
extern crate bitflags;

extern crate hal;


use hal::{board::MAX_PROCESSORS, constant::{Constant, ConstantsHal}, define_entry, instruction::{Instruction, InstructionHal}, pagetable::PageTableHal, println};
use log::*;
use mm::{vm::KernVmSpaceHal, KVMSPACE};
use processor::processor::current_processor;

#[allow(unused)]
mod net;
mod config;
mod banner;
mod devices;
mod drivers;
pub mod fs;
pub mod lang_items;
/// ipc
pub mod ipc;
pub mod mm;
//pub mod sbi;
pub mod sync;
pub mod syscall;
pub mod signal;
pub mod task;
mod processor;
pub mod timer;
pub mod trap;
mod executor;
pub mod utils;

use core::{arch::{global_asm, naked_asm}, sync::atomic::{AtomicBool,Ordering}};

use crate::timer::timer::TIMER_MANAGER;

/// id is the running processor, now start others
#[allow(unused)]
fn processor_start(id: usize) {
    use crate::processor::processor::PROCESSORS;
    let nums = MAX_PROCESSORS;
    for i in 0..nums {
        if i == id {
            continue;
        }
        Instruction::hart_start(i, 0);
        // info!("[kernel] start to wake up processor {}... ",i);
    }
}

/// the rust entry-point of os
/// return true if need reboot (but not supported yet)
fn main(id: usize, first: bool) -> bool {
    info!("into main");
    if first {
        info!("id: {id}");
        banner::print_banner();
        processor::processor::init(id);
        hal::trap::init();       
        devices::init();
        fs::init();
        // fs::vfs::file::list_apps(); 
        // fs::ext4::page_cache_test();       
        #[cfg(not(feature = "smp"))]
        executor::init();
        task::schedule::spawn_kernel_task(
            async move{
                task::add_initproc();
            }
        );

        #[cfg(feature = "smp")]
        processor_start(id);
    } else {
        processor::processor::init(id);
        hal::trap::init();
    }
    info!("[kernel] -------hart {} start-------",id);
    unsafe { 
        Instruction::enable_timer_interrupt();
    }
    timer::set_next_trigger();
    executor::run_until_shutdown();
    // return false: HAL will shutdown
    false
}

fn enable_kvm() {
    KVMSPACE.lock().enable();
}

#[cfg(target_arch="loongarch64")]
fn pre_main(id: usize, first: bool) -> bool {
    if first {
        mm::init();
    } else {
        enable_kvm();
    }
    main(id, first)
}

#[cfg(target_arch="riscv64")]
fn pre_main(id: usize, first: bool) -> bool {
    if first {
        mm::init();
    } else {
        enable_kvm();
    }
    main(id, first)
}



hal::define_entry!(pre_main);