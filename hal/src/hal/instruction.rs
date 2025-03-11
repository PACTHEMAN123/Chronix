
pub trait InstructionHal {

    unsafe fn tlb_flush_addr(va: usize);

    unsafe fn tlb_flush_all();

    unsafe fn disable_interrupt();

    unsafe fn enable_interrupt();

}

/// arch must impl InstructionHal for InstructionHal
pub struct Instruction;
