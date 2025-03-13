//! Implementation of [`TaskContext`]
use alloc::sync::Arc;
use riscv::register::sstatus;

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
            riscv::register::sstatus::clear_sum();
        } else {
            riscv::register::sstatus::set_sum();
        }
    }
    /// Change sum flag to new value
    pub fn change_env(&self, new:&Self){
        unsafe{new.auto_sum();}
    }
    /// increase sum flag
    pub fn sum_inc(&mut self) {
        if self.sum_flag == 0 {
            unsafe {
                sstatus::set_sum();
            }
            self.sum_flag = 1;
        }
    }
    /// decrease sum flag
    pub fn sum_dec(&mut self) {
        if self.sum_flag == 1 {
            unsafe {
                sstatus::clear_sum();
            }
            self.sum_flag = 0;
        }
    }
}

/// RAII to guard sum flag
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