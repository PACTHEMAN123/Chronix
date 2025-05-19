use core::fmt::Debug;

use log::{info, warn};
use loongArch64::register::{self, estat::{Exception, Interrupt, Trap}};

use crate::{addr::{VirtAddr, VirtAddrHal, VirtPageNum}, allocator::FakeFrameAllocator, board::MAX_PROCESSORS, instruction::{Instruction, InstructionHal}, pagetable::{MapFlags, PTEFlags, PageTable, PageTableEntryHal, PageTableHal}, println};

use super::{FloatContextHal, TrapContextHal, TrapType, TrapTypeHal};

core::arch::global_asm!(include_str!("trap.S"));

pub(crate) static mut FP_REG_DIRTY: [bool; MAX_PROCESSORS] = [false; MAX_PROCESSORS];

impl TrapTypeHal for TrapType {
    fn get() -> Self {
        get_trap_type()
    }
    
    fn get_debug() -> (Self, usize) {
        (get_trap_type(), loongArch64::register::era::read().raw())
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TrapContext {
    /// user-to-kernel should save:
    /// general regs[0..31]
    pub(crate) r: [usize; 32], // 0 ~ 31
    /// CSR PRMD
    pub(crate) prmd: register::prmd::Prmd, // 32
    /// CSR era (sepc)
    pub(crate) era: usize, // 33

    /// kernel-to-user should save:
    pub(crate) kernel_ctx: KernelContext, // 34 ~ 46

    /// float registers
    pub(crate) user_fx: FloatContext, // 47 ~ 79
    /// used in multi_core
    pub(crate) stored: usize, // 80
    /// used for signal, when using SA_RESTART flag, need to restore last user arg0
    pub(crate) last_user_arg0: usize, // 81
}

impl Debug for TrapContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrapContext").field("r", &self.r).field("era", &self.era).field("kernel_ctx", &self.kernel_ctx).field("user_fx", &self.user_fx).field("stored", &self.stored).finish()
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct KernelContext {
    /// stack point
    pub(crate) sp: usize, // 0
    /// return address
    pub(crate) ra: usize, // 1
    /// static registers
    pub(crate) s: [usize; 9], // 2 ~ 10
    /// frame pointer
    pub(crate) fp: usize, // 11
    /// thread pointer
    pub(crate) tp: usize, // 12
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FloatContext {
    pub(crate) f: [f64; 32], // 0 ~ 31
    pub(crate) fcsr: u32, // 32
    pub(crate) need_save: u8, // 32
    pub(crate) need_restore: u8, // 32
    pub(crate) signal_dirty: u8, // 32
}

impl TrapContextHal for TrapContext {
    fn syscall_id(&self) -> usize {
        self.r[11]
    }

    fn syscall_arg_nth(&self, n: usize) -> usize {
        assert!(n < 6);
        self.r[4 + n]
    }

    fn arg_nth(&self, n: usize) -> usize {
        if n < 8 {
            self.r[4 + n]
        } else {
            panic!("unsupported arguments number")
        }
    }

    fn set_arg_nth(&mut self, n: usize, arg: usize) {
        if n < 8 {
            self.r[4 + n] = arg
        } else {
            panic!("unsupported arguments number")
        }
    }

    fn ret_nth(&self, n: usize) -> usize {
        if n < 2 {
            self.r[4 + n]
        } else {
            panic!("unsupported return number")
        }
    }

    fn set_ret_nth(&mut self, n: usize, ret: usize) {
        if n < 2 {
            self.r[4 + n] = ret;
        } else {
            panic!("unsupported return number")
        }
    }

    fn ra(&mut self) -> &mut usize {
        &mut self.r[1]
    }

    fn sp(&mut self) -> &mut usize {
        &mut self.r[3]
    }

    fn tp(&mut self) -> &mut usize {
        &mut self.r[2]
    }

    fn sepc(&mut self) -> &mut usize {
        &mut self.era
    }

    fn app_init_context(entry: usize, sp: usize, argc: usize, argv: usize, envp: usize) -> Self {
        // set CPU privilege to User after trapping back
        unsafe {
            register::prmd::set_pplv(register::CpuMode::Ring3);
            Instruction::disable_interrupt();
        }
        let mut cs = Self {
            r: [0usize; 32],
            prmd: register::prmd::read(),
            era: entry,
            kernel_ctx: KernelContext::new(),
            user_fx: FloatContext::new(),
            stored: 0,
            last_user_arg0: 0,
        };
        *cs.sp() = sp;
        cs.set_arg_nth(0, argc);
        cs.set_arg_nth(1, argv);
        cs.set_arg_nth(2, envp);
        cs
    }

    fn save_to(&mut self, idx: usize, v: usize) {
        assert!(idx == 0);
        self.stored = v;
    }

    fn load_from(&mut self, idx: usize) -> usize {
        assert!(idx == 0);
        self.stored
    }

    fn mark_fx_save(&mut self) {
        self.user_fx.need_save |= register::euen::read().fpe() as u8;
        self.user_fx.signal_dirty |= register::euen::read().fpe() as u8;
    }

    fn fx_yield_task(&mut self) {
        self.user_fx.yield_task();
    }

    fn fx_encounter_signal(&mut self) {
        self.user_fx.encounter_signal();
    }

    fn fx_restore(&mut self) {
        self.user_fx.restore();
    }

    fn save_last_user_arg0(&mut self) {
        self.last_user_arg0 = self.r[4];
    }

    fn restore_last_user_arg0(&mut self) {
        self.r[4] = self.last_user_arg0;
    }
}

impl FloatContextHal for FloatContext {
    fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    fn save(&mut self) {
        if self.need_save == 0 {
            return;
        }
        self.need_save = 0;
        //warn!("FP save");
        let last_fpe = register::euen::read().fpe();
        register::euen::set_fpe(true);
        unsafe {
            let mut _t: usize = 1; // as long as not x0
            core::arch::asm!("
                fst.d  $f0, {0},  0*8
                fst.d  $f1, {0},  1*8
                fst.d  $f2, {0},  2*8
                fst.d  $f3, {0},  3*8
                fst.d  $f4, {0},  4*8
                fst.d  $f5, {0},  5*8
                fst.d  $f6, {0},  6*8
                fst.d  $f7, {0},  7*8
                fst.d  $f8, {0},  8*8
                fst.d  $f9, {0},  9*8
                fst.d $f10, {0}, 10*8
                fst.d $f11, {0}, 11*8
                fst.d $f12, {0}, 12*8
                fst.d $f13, {0}, 13*8
                fst.d $f14, {0}, 14*8
                fst.d $f15, {0}, 15*8
                fst.d $f16, {0}, 16*8
                fst.d $f17, {0}, 17*8
                fst.d $f18, {0}, 18*8
                fst.d $f19, {0}, 19*8
                fst.d $f20, {0}, 20*8
                fst.d $f21, {0}, 21*8
                fst.d $f22, {0}, 22*8
                fst.d $f23, {0}, 23*8
                fst.d $f24, {0}, 24*8
                fst.d $f25, {0}, 25*8
                fst.d $f26, {0}, 26*8
                fst.d $f27, {0}, 27*8
                fst.d $f28, {0}, 28*8
                fst.d $f29, {0}, 29*8
                fst.d $f30, {0}, 30*8
                fst.d $f31, {0}, 31*8
                movfcsr2gr {1}, $fcsr0
                st.w  {1}, {0}, 32*8
            ", 
            in(reg) self,
            inout(reg) _t
            );
        }
        register::euen::set_fpe(last_fpe);
    }

    fn restore(&mut self) {
        if self.need_restore == 0 {
            return;
        }
        self.need_restore = 0;
        //warn!("FP restore");
        let last_fpe = register::euen::read().fpe();
        register::euen::set_fpe(true);
        unsafe {
            let mut _t: usize = 1; // as long as not x0
            core::arch::asm!("
                fld.d  $f0, {0},  0*8
                fld.d  $f1, {0},  1*8
                fld.d  $f2, {0},  2*8
                fld.d  $f3, {0},  3*8
                fld.d  $f4, {0},  4*8
                fld.d  $f5, {0},  5*8
                fld.d  $f6, {0},  6*8
                fld.d  $f7, {0},  7*8
                fld.d  $f8, {0},  8*8
                fld.d  $f9, {0},  9*8
                fld.d $f10, {0}, 10*8
                fld.d $f11, {0}, 11*8
                fld.d $f12, {0}, 12*8
                fld.d $f13, {0}, 13*8
                fld.d $f14, {0}, 14*8
                fld.d $f15, {0}, 15*8
                fld.d $f16, {0}, 16*8
                fld.d $f17, {0}, 17*8
                fld.d $f18, {0}, 18*8
                fld.d $f19, {0}, 19*8
                fld.d $f20, {0}, 20*8
                fld.d $f21, {0}, 21*8
                fld.d $f22, {0}, 22*8
                fld.d $f23, {0}, 23*8
                fld.d $f24, {0}, 24*8
                fld.d $f25, {0}, 25*8
                fld.d $f26, {0}, 26*8
                fld.d $f27, {0}, 27*8
                fld.d $f28, {0}, 28*8
                fld.d $f29, {0}, 29*8
                fld.d $f30, {0}, 30*8
                fld.d $f31, {0}, 31*8
                ld.w  {1}, {0}, 32*8
                movgr2fcsr $fcsr0, {1}
            ", 
            in(reg) self,
            inout(reg) _t
            );
        }
        register::euen::set_fpe(last_fpe);
    }

    fn yield_task(&mut self) {
        self.save();
        self.need_restore = 1;
    }

    fn encounter_signal(&mut self) {
        self.save();
    }
}

impl KernelContext {
    fn new() -> Self {
        Self { sp: 0, ra: 0, s: [0usize; 9], fp: 0, tp: 0 }
    }
}

pub fn init() {
    set_kernel_trap_entry();
}


pub fn set_kernel_trap_entry() {
    unsafe extern "C" {
        fn __trap_from_kernel();
    }
    register::eentry::set_eentry(__trap_from_kernel as usize);
}

pub fn set_user_trap_entry() {
    unsafe extern "C" {
        fn __trap_from_user();
    }
    register::eentry::set_eentry(__trap_from_user as usize);
}

fn handle_page_modify_fault(badv: usize) -> TrapType {
    let va = VirtAddr(badv); //虚拟地址
    let vpn: VirtPageNum = va.floor(); //虚拟地址的虚拟页号
    let token = register::pgdl::read().base();
    let page_table = PageTable::<FakeFrameAllocator>::from_token(token, FakeFrameAllocator);
    let (pte, _) = page_table.find_pte(vpn).unwrap(); //获取页表项
    if !pte.flags().contains(MapFlags::W) {
        return TrapType::StorePageFault(badv);
    }
    pte.set_dirty(true);
    unsafe {
        core::arch::asm!("tlbsrch", "tlbrd",); //根据TLBEHI的虚双页号查询TLB对应项
    }
    let tlbidx = register::tlbidx::read(); //获取TLB项索引
    assert_eq!(tlbidx.ne(), false);
    register::tlbelo0::set_dirty(true);
    register::tlbelo1::set_dirty(true);

    unsafe {
        core::arch::asm!("tlbwr"); //重新将tlbelo写入tlb
    }
    
    TrapType::Processed
}

fn get_trap_type() -> TrapType {
    let estat = register::estat::read();
    let badv = register::badv::read().raw();
    match estat.cause() {
        Trap::Exception(Exception::Breakpoint) => TrapType::Breakpoint,
        Trap::Exception(Exception::Syscall) => TrapType::Syscall,
        Trap::Exception(Exception::LoadPageFault) => TrapType::LoadPageFault(badv),
        Trap::Exception(Exception::StorePageFault) => TrapType::StorePageFault(badv),
        Trap::Exception(Exception::FetchPageFault) => TrapType::InstructionPageFault(badv),
        Trap::Interrupt(Interrupt::Timer) => TrapType::Timer,
        Trap::Exception(Exception::PageModifyFault) => {
            handle_page_modify_fault(badv)
        },
        Trap::Exception(Exception::FloatingPointUnavailable) => {
            let cpuid = register::cpuid::read().core_id();
            unsafe { FP_REG_DIRTY[cpuid] = true; }
            TrapType::Processed
        },
        _ => {
            warn!(
                "TrapType::Other cause: {:?} badv: {:#x} badi: {:#x} era: {:#x}", 
                estat.cause(), 
                badv, 
                register::badi::read().inst(),
                register::era::read().raw()
            );
            TrapType::Other
        }
    }
}

pub fn restore(cx: &mut TrapContext) {
    unsafe extern "C" {
        fn __restore(cx: usize);
    }
    unsafe { 
        __restore(cx as *mut _ as _);
    }
}
