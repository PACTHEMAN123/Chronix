#![no_std]

#![feature(alloc_error_handler)]
#![feature(step_trait)]
#![feature(new_range_api)]
#![feature(naked_functions)]
#![feature(sync_unsafe_cell)]
#![feature(allocator_api)]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod component;

pub mod board;
pub mod util;

pub use component::*;

mod interface;

pub use interface::*;
