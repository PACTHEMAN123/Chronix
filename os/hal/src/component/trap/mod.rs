#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    Other,
    Processed,
    Breakpoint,
    Syscall,
    Timer,
    StorePageFault(usize),
    LoadPageFault(usize),
    InstructionPageFault(usize),
    IllegalInstruction(usize),
}

pub trait TrapTypeHal: Sized {
    fn get() -> Self;

    fn get_debug() -> (Self, usize);
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

    fn tp(&mut self) -> &mut usize;

    fn sepc(&mut self) -> &mut usize;

    fn app_init_context(entry: usize, sp: usize, argc: usize, argv: usize, envp: usize) -> Self;

    fn save_to(&mut self, idx: usize, v: usize);

    fn load_from(&mut self, idx: usize) -> usize;

    fn mark_fx_save(&mut self);

    fn fx_yield_task(&mut self);

    fn fx_encounter_signal(&mut self);

    fn fx_restore(&mut self);

    fn save_last_user_arg0(&mut self);

    fn restore_last_user_arg0(&mut self);
} 
pub trait FloatContextHal {
    fn new() -> Self;

    fn save(&mut self);

    fn restore(&mut self);

    fn yield_task(&mut self);

    fn encounter_signal(&mut self);
}

#[macro_export]
macro_rules! define_kernel_trap_handler {
    ($fn: ident) => {
        /// hal_kernel_trap_handler_for_arch
        #[unsafe(export_name = "kernel_trap_handler")]
        pub fn __hal_kernel_trap_handler() {
            $fn()
        }
    };
}

#[macro_export]
macro_rules! define_user_trap_handler {
    ($fn: ident) => {
        /// hal_user_trap_handler
        #[unsafe(export_name = "user_trap_handler")]
        pub async fn __hal_user_trap_handler() {
            $fn().await;
        }
    };
}


#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
#[allow(unused)]
pub use loongarch64::*;
