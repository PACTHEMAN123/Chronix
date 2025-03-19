#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    Other,
    Breakpoint,
    Syscall,
    Timer,
    StorePageFault(usize),
    LoadPageFault(usize),
    InstructionPageFault(usize),
    IllegalInstruction(usize),
}

pub trait TrapContextHal {
    fn syscall_id(&self) -> usize;

    fn syscall_arg_nth(&self, n: usize) -> usize;

    fn arg_nth(&self, n: usize) -> usize;

    fn set_arg_nth(&mut self, n: usize, arg: usize);

    fn ret_nth(&self, n: usize) -> usize;

    fn set_ret_nth(&mut self, n: usize, ret: usize);

    fn ra(&mut self) -> &mut usize;

    fn sp(&mut self) -> &mut usize;

    fn sepc(&mut self) -> &mut usize;

    fn tls(&mut self) -> &mut usize;

    fn app_init_context(entry: usize, sp: usize) -> Self;

    fn save_to(&mut self, idx: usize, v: usize);

    fn load_from(&mut self, idx: usize) -> usize;

    fn mark_fx_save(&mut self);

    fn fx_yield_task(&mut self);

    fn fx_encounter_signal(&mut self);

    fn fx_restore(&mut self);

} 
pub trait FloatContextHal {
    fn new() -> Self;

    fn save(&mut self);

    fn restore(&mut self);

    fn yield_task(&mut self);

    fn encounter_signal(&mut self);
}
unsafe extern "Rust" {
    fn kernel_trap_handler_for_arch(trap_type: TrapType);
}

#[macro_export]
macro_rules! define_kernel_trap_handler {
    ($fn: ident) => {
        /// hal_kernel_trap_handler_for_arch
        #[unsafe(export_name = "kernel_trap_handler_for_arch")]
        pub fn hal_kernel_trap_handler_for_arch(trap_type: TrapType) {
            $fn(trap_type)
        }
    };
}


#[cfg(target_arch = "riscv64")]
mod riscv64;

use core::usize;

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
#[allow(unused)]
pub use loongarch64::*;
