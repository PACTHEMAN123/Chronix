// From https://github.com/Godones/rCoreloongArch

use core::ops::{Deref, DerefMut};

use loongArch64::iocsr::{iocsr_read_b, iocsr_read_d, iocsr_read_h, iocsr_read_w, iocsr_write_b, iocsr_write_d, iocsr_write_h, iocsr_write_w};

use crate::{irq::IrqCtrlHal, println};


pub const LOONGARCH_IOCSR_EXTIOI_EN_BASE: usize = 0x1600; //扩展 IO 中断[63:0]的中断使能配置
pub const LOONGARCH_IOCSR_EXTIOI_ISR_BASE: usize = 0x1800; //路由至处理器核 0 的扩展 IO 中断[63:0]的中断状态
pub const LOONGARCH_IOCSR_EXTIOI_MAP_BASE: usize = 0x14c0; //EXT_IOI[31:0]的引脚路由方式
pub const LOONGARCH_IOCSR_EXTIOI_ROUTE_BASE: usize = 0x1c00; //EXT_IOI[0]的处理器核路由方式
pub const LOONGARCH_IOCSR_EXRIOI_NODETYPE_BASE: usize = 0x14a0; //16 个结点的映射向量类型 0（软件配置
pub const LOONGARCH_IOCSR_EXRIOI_SEND: usize = 0x1140; // 配置寄存器中增加了一个扩展 IO 中断触发寄存
                                                       // 器，用于将对应的 IO 中断置位

/// 初始化外部中断
pub fn extioi_init() {
    // 使能外部设备中断
    iocsr_write_d(LOONGARCH_IOCSR_EXTIOI_EN_BASE, 0);
    // extioi[31:0] map to cpu irq pin INT1, other to INT0
    //路由到INT1上
    iocsr_write_b(LOONGARCH_IOCSR_EXTIOI_MAP_BASE, 0x1);
    // extioi IRQ 0-7 route to core 0, use node type 0
    //路由到EXT_IOI_node_type0指向的0号处理器上
    iocsr_write_w(LOONGARCH_IOCSR_EXTIOI_ROUTE_BASE, 0x0);
    // nodetype0 set to 1, always trigger at node 0 */
    //固定分发模式时,只在0号处理器上触发
    iocsr_write_h(LOONGARCH_IOCSR_EXRIOI_NODETYPE_BASE, 0x1);
    //检查扩展i/o触发器是不是全0，即没有被触发的中断
    let extioi_isr = iocsr_read_b(LOONGARCH_IOCSR_EXTIOI_ISR_BASE);
    println!("extioi_init: extioi_isr = {:#b}", extioi_isr);
    let current_trigger = extioi_claim();
    println!("extioi_init: current_trigger = {:#b}", current_trigger);
    assert_eq!(extioi_isr, 0);
}

// ask the extioi what interrupt we should serve.
pub fn extioi_claim() -> u64 {
    iocsr_read_d(LOONGARCH_IOCSR_EXTIOI_ISR_BASE)
}

pub fn extioi_complete(irq: u64) {
    iocsr_write_d(LOONGARCH_IOCSR_EXTIOI_ISR_BASE, 1 << irq);
}

pub const LS7A_PCH_REG_BASE: usize = 0x1000_0000 + 0x8000_0000_0000_0000;
pub const LS7A_INT_MASK_REG: usize = LS7A_PCH_REG_BASE + 0x020; //中断掩码寄存器低32位
pub const LS7A_INT_EDGE_REG: usize = LS7A_PCH_REG_BASE + 0x060; //触发方式寄存器
pub const LS7A_INT_CLEAR_REG: usize = LS7A_PCH_REG_BASE + 0x080; //边沿触发中断清除寄存器
pub const LS7A_INT_HTMSI_VEC_REG: usize = LS7A_PCH_REG_BASE + 0x200; //HT 中断向量寄存器[ 7- 0]
pub const LS7A_INT_STATUS_REG: usize = LS7A_PCH_REG_BASE + 0x3a0; //中断状态（在服务）寄存器 ISR
pub const LS7A_INT_POL_REG: usize = LS7A_PCH_REG_BASE + 0x3e0; //中断触发电平选择寄存器

pub fn ls7a_read_w(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_volatile() }
}

pub fn ls7a_write_w(addr: usize, value: u32) {
    unsafe {
        (addr as *mut u32).write_volatile(value);
    }
}
pub fn ls7a_write_b(addr: usize, value: u8) {
    unsafe {
        (addr as *mut u8).write_volatile(value);
    }
}
pub fn ls7a_read_b(addr: usize) -> u8 {
    unsafe { (addr as *const u8).read_volatile() }
}

pub fn ls7a_read_d(addr: usize) -> u64 {
    unsafe { (addr as *const u64).read_volatile() }
}

pub fn ls7a_write_d(addr: usize, value: u64) {
    unsafe {
        (addr as *mut u64).write_volatile(value);
    }
}

/// 初始化ls7a中断控制器
pub fn ls7a_intc_init() {
    ls7a_write_w(
        LS7A_INT_MASK_REG, !0x0
    );
    // 触发方式设置寄存器
    // 0：电平触发中断
    // 1：边沿触发中断
    // 这里设置为电平触发
    ls7a_write_w(
        LS7A_INT_EDGE_REG,
        0x0,
    );
    // route to the same irq in extioi, pch_irq == extioi_irq
    // ls7a_write_b(LS7A_INT_HTMSI_VEC_REG + UART0_IRQ, UART0_IRQ as u8);
    // ls7a_write_b(LS7A_INT_HTMSI_VEC_REG + KEYBOARD_IRQ, KEYBOARD_IRQ as u8);
    // ls7a_write_b(LS7A_INT_HTMSI_VEC_REG + MOUSE_IRQ, MOUSE_IRQ as u8);
    // 设置中断电平触发极性
    // 对于电平触发类型：
    // 0：高电平触发；
    // 1：低电平触发
    // 这里是高电平触发
    ls7a_write_w(LS7A_INT_POL_REG, 0x0);
}

pub fn ls7a_intc_complete(irq: u64) {
    // 将对应位写1 清除中断
    ls7a_write_d(LS7A_INT_CLEAR_REG, 1 << irq);
}

pub struct IrqCtrl;

impl IrqCtrlHal for IrqCtrl {
    fn from_dt(device_tree: &fdt::Fdt, mmio: impl crate::mapper::MmioMapperHal) -> Option<Self> where Self: Sized {
        extioi_init();
        ls7a_intc_init();
        Some(IrqCtrl)
    }

    fn enable_irq(&self, no: usize, _: usize) {
        
        iocsr_write_d(LOONGARCH_IOCSR_EXTIOI_EN_BASE, 
            iocsr_read_d(LOONGARCH_IOCSR_EXTIOI_EN_BASE) | 0x1 << no
        );

        ls7a_write_w(
            LS7A_INT_MASK_REG,
            ls7a_read_w(LS7A_INT_MASK_REG) & !(0x1 << no)
        );

        ls7a_write_w(
            LS7A_INT_EDGE_REG,
            ls7a_read_w(LS7A_INT_EDGE_REG) | 0x1 << no
        );
        ls7a_write_b(LS7A_INT_HTMSI_VEC_REG + no, no as u8);
    }

    fn disable_irq(&self, no: usize, _: usize) {
    }

    fn claim_irq(&self, _: usize) -> Option<usize> {
        let val = extioi_claim();
        let no = 63 - val.leading_zeros();
        Some(no as usize)
    }

    fn complete_irq(&self, no: usize, _: usize) {
        assert!(no < 64);
        extioi_complete(no as u64);
        ls7a_intc_complete(no as u64);
    }
}