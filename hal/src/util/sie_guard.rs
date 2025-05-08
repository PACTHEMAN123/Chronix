
use crate::component::instruction::{Instruction, InstructionHal};
/// Sie Guard
pub struct SieGuard(bool);

impl SieGuard {
    /// Construct a SieGuard
    pub fn new() -> Self {
        Self(unsafe {
            let sie_before = Instruction::is_interrupt_enabled();
            Instruction::disable_interrupt();
            sie_before
        })
    }
}
impl Drop for SieGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                Instruction::enable_interrupt();
            }
        }
    }
}