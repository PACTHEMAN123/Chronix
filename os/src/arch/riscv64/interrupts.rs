use riscv::register::{
    sie, sstatus,
    stvec::{self, TrapMode},
};

pub unsafe fn disable_interrupt() {
    sstatus::clear_sie();
}
#[allow(dead_code)]
pub unsafe fn enable_interrupt() {
    sstatus::set_sie();
}