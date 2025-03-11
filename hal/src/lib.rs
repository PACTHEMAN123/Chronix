#![no_std]
#![no_main]

#![feature(step_trait)]
#![feature(new_range_api)]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod hal;
mod arch;

pub use hal::{instruction, mem, trap};
pub use arch::*;