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
#![deny(missing_docs)]
#![deny(warnings)]
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

extern crate alloc;

#[macro_use]
extern crate bitflags;

use board::MAX_PROCESSORS;
use log::*;
use mm::vm::{VmSpace, KERNEL_SPACE};
use processor::processor::current_processor;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use] 
mod console;
mod config;
mod devices;
mod drivers;
pub mod fs;
pub mod lang_items;
mod logging;
pub mod mm;
pub mod sbi;
pub mod sync;
pub mod syscall;
pub mod signal;
pub mod task;
mod processor;
pub mod timer;
pub mod trap;
mod arch;
mod executor;
mod async_utils;

use core::{arch::global_asm, sync::atomic::{AtomicBool,Ordering}};

// global_asm!(include_str!("entry.asm"));
/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}
#[allow(unused)]
static FIRST_PROCESSOR: AtomicBool = AtomicBool::new(true);
/// id is the running processor, now start others
#[allow(unused)]
fn processor_start(id: usize) {
    use crate::config::KERNEL_ENTRY_PA;
    use crate::processor::processor::PROCESSORS;
    let nums = MAX_PROCESSORS;
    for i in 0..nums {
        if i == id {
            continue;
        }
        let status = sbi_rt::hart_start(i, KERNEL_ENTRY_PA,0);
        //info!("[kernel] start to wake up processor {}... status {:?}",i,status);
    }
}

#[no_mangle]
/// the rust entry-point of os
pub fn rust_main(id: usize) -> ! {
    if FIRST_PROCESSOR.load(Ordering::Acquire)
    {
        FIRST_PROCESSOR.store(false, Ordering::Release);
        clear_bss();
        logging::init();
        info!("id: {id}");
        info!("[kernel] Hello, world!");
        mm::init();
        mm::vm::remap_test();
        processor::processor::init(id);
        trap::init();
        fs::init();
        fs::ext4::list_apps();        
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
        trap::init();
        KERNEL_SPACE.exclusive_access().enable();
    }
    info!("[kernel] -------hart {} start-------",id);
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    loop{
        //info!("now Idle loop");
       let _tasks = executor::run_until_idle();
       //info!("[kernel] {} have {} tasks run",current_processor().id(),tasks);
    }
    //panic!("Unreachable in rust_main!");
}
