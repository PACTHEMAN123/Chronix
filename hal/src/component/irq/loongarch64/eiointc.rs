use loongArch64::iocsr::{iocsr_read_d, iocsr_write_b, iocsr_write_d, iocsr_write_h};

/// Extended IO Interrupt Controller
pub struct Eiointc;

impl Eiointc {
    /// 扩展 IO 中断使能寄存器, 4个64位，每个控制64个中断
    pub const EXT_IOI_EN_BASE: usize = 0x1600;
    /// 扩展 IO 中断自动轮转使能寄存器, 4个64位，每个控制64个中断
    pub const EXT_IOI_BOUNCE_BASE: usize = 0x1680;
    /// 扩展 IO 中断状态寄存器, 4个64位，每个控制64个中断
    pub const EXT_IOI_SR_BASE: usize = 0x1700;
    /// 当前处理器核的扩展 IO 中断状态寄存器，4个64位，每个表示64个中断
    pub const PERCORE_EXT_IOI_SR_BASE: usize = 0x1800;
    /// 中断引脚路由寄存器，8个8位，每个控制32个中断的中断引脚
    pub const EXT_IOI_MAP_BASE: usize = 0x14C0;
    /// 中断目标处理器核路由寄存器，256个8位，每个控制一个中断的中断目标处理器核
    pub const EXT_IOI_MAP_CORE_BASE: usize = 0x1C00;
    /// 中断目标结点映射方式配置，16个16位
    pub const EXT_IOI_NODE_TYPE_BASE: usize = 0x14A0;

    /// 初始化中断路由，不支持多cpu，最多4个cpu核心
    pub fn init(core_num: usize) {
        assert!(core_num <= 4);
        let mut v = iocsr_read_d(0x420);
        v |= 1 << 49;
        iocsr_write_d(0x420, v);
        // 中断号N的中断，路由到N/32号引脚
        // 0..31    -> INT0
        // 32..63   -> INT1
        // 64..95   -> INT2
        // 96..127  -> INT3
        // 128..159 -> INT4
        // 160..191 -> INT5
        // 192..223 -> INT6
        // 224..255 -> INT7
        for i in 0..8 {
            iocsr_write_b(Self::EXT_IOI_MAP_BASE + i as usize, i);
        }
        // 每个中断在第一个映射方式寄存器指向的结点的n个核间轮转
        for i in 0..256 {
            iocsr_write_b(Self::EXT_IOI_MAP_CORE_BASE + i as usize, (1 << core_num) - 1);
        }
        // 第一个结点映射方式寄存器指向结点0
        iocsr_write_h(Self::EXT_IOI_NODE_TYPE_BASE, 0x01);
    }

    /// 使能对应中断
    pub fn enable_irq(no: usize) {
        let gp64 = no >> 6;
        let off64 = no & 63;
        // 使能中断
        let mut en = iocsr_read_d(Self::EXT_IOI_EN_BASE + gp64*8);
        en |= 1u64 << off64;
        iocsr_write_d(Self::EXT_IOI_EN_BASE + gp64*8, en);
        // 启用自动轮转
        let mut bounce = iocsr_read_d(Self::EXT_IOI_BOUNCE_BASE + gp64*8);
        bounce |= 1u64 << off64;
        iocsr_write_d(Self::EXT_IOI_BOUNCE_BASE + gp64*8, bounce);
    }

    /// 关闭对应中断
    pub fn disable_irq(no: usize) {
        let group = no >> 6;
        let idx = no & 63;
        let gp_offset = group << 3;
        let mut en = iocsr_read_d(Self::EXT_IOI_EN_BASE + gp_offset);
        en &= !(1u64 << idx);
        iocsr_write_d(Self::EXT_IOI_EN_BASE + gp_offset, en);
    }

    /// 获取路由到当前cpu核的中断号
    pub fn claim_irq() -> Option<usize> {
        let flags3 = iocsr_read_d(Self::PERCORE_EXT_IOI_SR_BASE + 3*8);
        if flags3 != 0 {
            return Some(255 - flags3.leading_zeros() as usize);
        }
        let flags2 = iocsr_read_d(Self::PERCORE_EXT_IOI_SR_BASE + 2*8);
        if flags2 != 0 {
            return Some(191 - flags2.leading_zeros() as usize);
        }
        let flags1 = iocsr_read_d(Self::PERCORE_EXT_IOI_SR_BASE + 1*8);
        if flags1 != 0 {
            return Some(127 - flags1.leading_zeros() as usize);
        }
        let flags0 = iocsr_read_d(Self::PERCORE_EXT_IOI_SR_BASE);
        if flags0 != 0 {
            return Some(63 - flags0.leading_zeros() as usize);
        }
        None
    }

    /// 中断处理完毕
    pub fn complete_irq(no: usize)  {
        let group = no >> 6;
        let idx = no & 63;
        let gp_offset = group << 3;
        let mut en = iocsr_read_d(Self::EXT_IOI_SR_BASE + gp_offset);
        en &= !(1u64 << idx);
        iocsr_write_d(Self::EXT_IOI_SR_BASE + gp_offset, en);
    }
}