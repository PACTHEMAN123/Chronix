use core::arch::asm;

use log::info;
use riscv::register::{scause::{self, Exception, Interrupt, Trap}, sepc, sstatus::{self, Sstatus, FS, SPP}, stval, stvec::{self, TrapMode}};

use crate::instruction::{Instruction, InstructionHal};

use super::{FloatContextHal, TrapContextHal, TrapType, TrapTypeHal};

core::arch::global_asm!(include_str!("trap.S"));


impl TrapTypeHal for TrapType {
    fn get() -> Self {
        get_trap_type()
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapContext {
    /// user-to-kernel should save:
    /// general regs[0..31]
    pub(crate) x: [usize; 32],
    /// CSR sstatus      
    pub(crate) sstatus: Sstatus, // 32
    // pub sstatus: usize, // 32
    /// CSR sepc
    pub(crate) sepc: usize, // 33

    /// Unlike rCore-tutorial, we don't need to save
    /// trap_handler here, since we will trap back to kernel
    /// and go to trap handler by reloading kernel's ra(through __trap_from_user).
    // pub trap_handler: usize,

    /// kernel-to-user should save:
    ///
    pub(crate) kernel_sp: usize, // 34
    ///
    pub(crate) kernel_ra: usize, // 35
    ///
    pub(crate) kernel_s: [usize; 12], // 36 - 47
    ///
    pub(crate) kernel_fp: usize, // 48
    ///
    pub(crate) kernel_tp: usize, // 49
    /// float registers
    pub(crate) user_fx: FloatContext, 
    /// used in multi_core
    pub(crate) stored: usize,
}

impl TrapContextHal for TrapContext {
    fn syscall_id(&self) -> usize {
        self.x[17]
    }

    fn syscall_arg_nth(&self, n: usize) -> usize {
        assert!(n < 6);
        self.x[10 + n]
    }

    fn arg_nth(&self, n: usize) -> usize {
        if n < 8 {
            self.x[10 + n]
        } else {
            todo!()
        }
    }

    fn set_arg_nth(&mut self, n: usize, arg: usize) {
        if n < 8 {
            self.x[10 + n] = arg
        } else {
            todo!()
        }
    }

    fn sp(&mut self) -> &mut usize {
        &mut self.x[2]
    }

    fn sepc(&mut self) -> &mut usize {
        &mut self.sepc
    }

    fn app_init_context(
        entry: usize,
        sp: usize,
        argc: usize,
        argv: usize,
        envp: usize,
    ) -> Self {
        // set CPU privilege to User after trapping back
        unsafe {
            sstatus::set_spp(SPP::User);
            Instruction::disable_interrupt();
        }
        let mut cx = Self {
            x: [0; 32],
            sstatus: sstatus::read(),
            sepc: entry,
            // saved in ___restore
            kernel_sp: 0,
            kernel_ra: 0,
            kernel_s: [0; 12],
            kernel_fp: 0,
            kernel_tp: 0,
            user_fx: FloatContext::new(),
            stored: 0,
        };
        *cx.sp() = sp;
        cx.set_arg_nth(0, argc);
        cx.set_arg_nth(1, argv);
        cx.set_arg_nth(2, envp);
        cx
    }
    
    fn ret_nth(&self, n: usize) -> usize {
        if n < 2 {
            self.x[10 + n]
        } else {
            todo!()
        }
    }
    
    fn set_ret_nth(&mut self, n: usize, ret: usize) {
        if n < 2 {
            self.x[10 + n] = ret;
        } else {
            todo!()
        }
    }
    
    fn save_to(&mut self, idx: usize, v: usize) {
        assert!(idx == 0);
        self.stored = v;
    }
    
    fn load_from(&mut self, idx: usize) -> usize {
        assert!(idx == 0);
        self.stored
    }
    
    fn tp(&mut self) -> &mut usize {
        &mut self.x[4]
    }
    
    fn ra(&mut self) -> &mut usize {
        &mut self.x[1]
    }
    
    fn mark_fx_save(&mut self) {
        self.user_fx.need_save |= (self.sstatus.fs() == FS::Dirty) as u8;
        self.user_fx.signal_dirty |= (self.sstatus.fs() == FS::Dirty) as u8;
    }
    
    fn fx_restore(&mut self) {
        self.user_fx.restore();
    }

    fn fx_yield_task(&mut self) {
        self.user_fx.yield_task();
    }

    fn fx_encounter_signal(&mut self){
        self.user_fx.encounter_signal();
    }

}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FloatContext {
    pub fx: [f64; 32],  // 50-81
    pub fcsr: u32,       
    pub need_save: u8,
    pub need_restore: u8,
    pub signal_dirty: u8,
}

impl FloatContextHal for FloatContext {
    fn new() -> Self{
        unsafe {core::mem::zeroed()}
    }
    fn save(&mut self) {
        if self.need_save == 0 {
            return;
        }
        self.need_save = 0;
        log::warn!("FP save");
        unsafe {
            let mut _t: usize = 1; // as long as not x0
            asm!("
            fsd  f0,  0*8({0})
            fsd  f1,  1*8({0})
            fsd  f2,  2*8({0})
            fsd  f3,  3*8({0})
            fsd  f4,  4*8({0})
            fsd  f5,  5*8({0})
            fsd  f6,  6*8({0})
            fsd  f7,  7*8({0})
            fsd  f8,  8*8({0})
            fsd  f9,  9*8({0})
            fsd f10, 10*8({0})
            fsd f11, 11*8({0})
            fsd f12, 12*8({0})
            fsd f13, 13*8({0})
            fsd f14, 14*8({0})
            fsd f15, 15*8({0})
            fsd f16, 16*8({0})
            fsd f17, 17*8({0})
            fsd f18, 18*8({0})
            fsd f19, 19*8({0})
            fsd f20, 20*8({0})
            fsd f21, 21*8({0})
            fsd f22, 22*8({0})
            fsd f23, 23*8({0})
            fsd f24, 24*8({0})
            fsd f25, 25*8({0})
            fsd f26, 26*8({0})
            fsd f27, 27*8({0})
            fsd f28, 28*8({0})
            fsd f29, 29*8({0})
            fsd f30, 30*8({0})
            fsd f31, 31*8({0})
            csrr {1}, fcsr
            sw  {1}, 32*8({0})
        ", in(reg) self,
           inout(reg) _t
            );
        };
    }
    fn yield_task(&mut self) {
        self.save();
        self.need_restore = 1;
    }

    fn encounter_signal(&mut self){
        self.save();
    }

    fn restore(&mut self) {
        if self.need_restore == 0 {
            return;
        }
        self.need_restore = 0;
        //log::warn!("FP restore");
        //println!("{:#x}", self as *mut Self as usize);
        unsafe {
            let mut _t: usize = 1; // as long as not x0
            asm!("
            fld  f0,  0*8({0})
            fld  f1,  1*8({0})
            fld  f2,  2*8({0})
            fld  f3,  3*8({0})
            fld  f4,  4*8({0})
            fld  f5,  5*8({0})
            fld  f6,  6*8({0})
            fld  f7,  7*8({0})
            fld  f8,  8*8({0})
            fld  f9,  9*8({0})
            fld f10, 10*8({0})
            fld f11, 11*8({0})
            fld f12, 12*8({0})
            fld f13, 13*8({0})
            fld f14, 14*8({0})
            fld f15, 15*8({0})
            fld f16, 16*8({0})
            fld f17, 17*8({0})
            fld f18, 18*8({0})
            fld f19, 19*8({0})
            fld f20, 20*8({0})
            fld f21, 21*8({0})
            fld f22, 22*8({0})
            fld f23, 23*8({0})
            fld f24, 24*8({0})
            fld f25, 25*8({0})
            fld f26, 26*8({0})
            fld f27, 27*8({0})
            fld f28, 28*8({0})
            fld f29, 29*8({0})
            fld f30, 30*8({0})
            fld f31, 31*8({0})
            lw  {1}, 32*8({0})
            csrw fcsr, {1}
        ", in(reg) self,
           inout(reg) _t
            );
        }
    }
}

pub fn init() {
    set_kernel_trap_entry();
}


pub fn set_kernel_trap_entry() {
    unsafe extern "C" {
        fn __trap_from_kernel();
    }
    unsafe {
        stvec::write(__trap_from_kernel as usize, TrapMode::Direct);
    }
}

pub fn set_user_trap_entry() {
    unsafe extern "C" {
        fn __trap_from_user();
    }
    unsafe {
        stvec::write(__trap_from_user as usize, TrapMode::Direct);
    }
}

fn get_trap_type() -> TrapType {
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => TrapType::Breakpoint,
        Trap::Exception(Exception::UserEnvCall) => TrapType::Syscall,
        Trap::Exception(Exception::LoadPageFault) => TrapType::LoadPageFault(stval),
        Trap::Exception(Exception::StorePageFault) => TrapType::StorePageFault(stval),
        Trap::Exception(Exception::InstructionPageFault) => TrapType::InstructionPageFault(stval),
        Trap::Interrupt(Interrupt::SupervisorTimer) => TrapType::Timer,
        _ => {
            info!("scause: {:?}, stval: {:x} sepc: {:x}", scause.cause(), stval, sepc::read());
            TrapType::Other
        }
    }
}

pub fn restore(cx: usize) {
    unsafe {
        core::arch::asm!(
            "call __restore",    
            in("a0") cx,      
        );
    }
}
