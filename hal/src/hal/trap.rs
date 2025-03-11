#[derive(Debug, Clone, Copy)]
pub enum TrapType {
    Breakpoint,
    SysCall,
    Timer,
    Unknown,
    SupervisorExternal,
    StorePageFault(usize),
    LoadPageFault(usize),
    InstructionPageFault(usize),
    IllegalInstruction(usize),
}

pub trait TrapContextHal {
    fn args(&self) -> &[usize];
    fn syscall(&self) -> usize;
    fn ra(&self) -> usize;
    fn sp(&self) -> usize;
}
