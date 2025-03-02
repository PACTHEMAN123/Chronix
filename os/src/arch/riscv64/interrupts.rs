use riscv::register::{
    sie, sstatus,
    stvec::{self, TrapMode},
};

pub unsafe fn disable_interrupt() {
    sstatus::clear_sie();
}

pub unsafe fn enable_interrupt() {
    sstatus::set_sie();
}