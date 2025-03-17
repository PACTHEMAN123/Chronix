use riscv::register::{scause::{self, Exception, Interrupt, Trap}, sstatus::{self, Sstatus, SPP}, stval, stvec::{self, TrapMode}};

use super::{TrapContextHal, TrapType};

core::arch::global_asm!(include_str!("trap.S"));


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
    /// used in multi_core
    pub(crate) stored: usize
}

impl TrapContextHal for TrapContext {
    fn syscall_id(&self) -> usize {
        self.x[17]
    }

    fn syscall_arg_nth(&self, n: usize) -> usize {
        assert!(n < 3);
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
    ) -> Self {
        // set CPU privilege to User after trapping back
        unsafe {
            sstatus::set_spp(SPP::User);
            sstatus::clear_sie();
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
            stored: 0,
        };
        *cx.sp() = sp;
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
    
    fn tls(&mut self) -> &mut usize {
        &mut self.x[4]
    }
    
    fn ra(&mut self) -> &mut usize {
        &mut self.x[1]
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

#[macro_export]
macro_rules! define_user_trap_handler {
    ($fn: ident) => {
        /// hal_user_trap_handler
        #[unsafe(export_name = "user_trap_handler")]
        pub async fn hal_user_trap_handler() {
            let scause = scause::read();
            let stval = stval::read();

            let trap_type = match scause.cause() {
                Trap::Exception(Exception::Breakpoint) => TrapType::Breakpoint,
                Trap::Exception(Exception::UserEnvCall) => TrapType::Syscall,
                Trap::Exception(Exception::LoadPageFault) => TrapType::LoadPageFault(stval),
                Trap::Exception(Exception::StorePageFault) => TrapType::StorePageFault(stval),
                Trap::Exception(Exception::InstructionPageFault) => TrapType::InstructionPageFault(stval),
                Trap::Interrupt(Interrupt::SupervisorTimer) => TrapType::Timer,
                _ => TrapType::Other
            };
            
            $fn(trap_type).await;
        }
    };
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
        _ => TrapType::Other
    }
}

#[unsafe(no_mangle)]
fn kernel_trap_handler() {
    unsafe { 
        super::kernel_trap_handler_for_arch(get_trap_type());
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
