//! Implementation of [`TaskContext`]
use alloc::sync::Arc;
use hal::instruction::{Instruction, InstructionHal};

use super::processor::current_processor;
/// sum flag guard
pub struct EnvContext {
    /// Permit supervisor user memory access
    sum_flag: usize,
}

impl EnvContext{
    /// Create a new [`EnvContext`]
    pub const fn new() -> Self {
        Self {
            sum_flag: 0,
        }
    }
    /// Auto set or clear sum flag
    pub unsafe fn auto_sum(&self) {
        log::trace!("[EnvContext::auto_sum] sum_cnt: {}", self.sum_flag);
        if self.sum_flag == 0 {
            Instruction::clear_sum();
        } else {
            Instruction::set_sum();
        }
    }
    /// Change sum flag to new value
    pub fn change_env(&self, new:&Self){
        unsafe{new.auto_sum();}
    }
    /// increase sum flag
    pub fn sum_inc(&mut self) {
        unsafe {
            Instruction::set_sum();
        }
        self.sum_flag += 1;
    }
    /// decrease sum flag
    pub fn sum_dec(&mut self) {
        self.sum_flag -= 1;
        if self.sum_flag == 0 {
            unsafe {
                Instruction::clear_sum();
            }
        }
    }
}

/// RAII to guard sum flag
#[repr(C)]
pub struct SumGuard {}

impl SumGuard{
    #[allow(dead_code)]
    /// Create a new [`SumGuard`]
    pub fn new() -> Self{
        current_processor().env_mut().sum_inc();
        Self{}
    }
}

impl Drop for SumGuard {
    fn drop(&mut self) {
        current_processor().env_mut().sum_dec();
    }
}

impl Clone for SumGuard {
    fn clone(&self) -> Self {
        Self::new()
    }
}