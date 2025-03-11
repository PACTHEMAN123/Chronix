#![no_std]

#![feature(alloc_error_handler)]
#![feature(step_trait)]
#![feature(new_range_api)]
#![feature(naked_functions)]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod component;

pub use component::*;

mod interface;

pub use interface::*;