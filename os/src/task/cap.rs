/// Linux 能力标志枚举 (对应 linux/capability.h)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// 覆盖文件所有权限制 (POSIX)
    ///
    /// 在启用 `_POSIX_CHOWN_RESTRICTED` 的系统上，可修改文件所有者和组
    Chown = 0,

    /// 覆盖所有DAC访问限制 (POSIX)
    ///
    /// 包括ACL执行权限（若启用 `_POSIX_ACL`)，除 `LINUX_IMMUTABLE` 保护的文件
    DacOverride = 1,

    /// 覆盖文件/目录的读和搜索限制 (POSIX)
    ///
    /// 包括ACL限制（若启用 `_POSIX_ACL`)，除 `LINUX_IMMUTABLE` 保护的文件
    DacReadSearch = 2,

    /// 绕过文件操作UID匹配限制 (POSIX)
    ///
    /// 覆盖要求文件所有者UID必须等于用户UID的限制（`CAP_FSETID` 适用的情况除外）
    Fowner = 3,

    /// 绕过setuid/setgid位设置限制 (POSIX)
    ///
    /// 覆盖以下限制：
    /// - 设置 setuid 位时需有效用户ID匹配文件所有者
    /// - 设置 setgid 位时需有效组ID匹配文件所有者
    Fsetid = 4,

    /// 绕过信号发送权限检查 (POSIX)
    ///
    /// 覆盖发送信号时需真实/有效用户ID匹配接收进程的限制
    Kill = 5,

    /// 允许操纵组身份 (POSIX)
    ///
    /// 允许使用 setgid(2), setgroups(2)
    /// 允许在套接字凭证传递中伪造GID
    Setgid = 6,

    /// 允许操纵用户身份 (POSIX)
    ///
    /// 允许使用 setuid(2) 系列函数（包括fsuid）
    /// 允许在套接字凭证传递中伪造PID
    Setuid = 7,

    /// 能力边界集操作 (Linux扩展)
    ///
    /// 允许：
    /// - 向当前进程的可继承集添加能力
    /// - 从能力边界集移除能力
    /// - 修改进程的安全位（securebits）
    Setpcap = 8,

    /// 修改不可变/只追加文件属性 (Linux扩展)
    ///
    /// 允许修改 S_IMMUTABLE 和 S_APPEND 文件属性
    LinuxImmutable = 9,

    /// 绑定低端口网络服务 (Linux扩展)
    ///
    /// 允许绑定1024以下的TCP/UDP套接字
    /// 允许绑定32以下的ATM VCI
    NetBindService = 10,

    /// 网络广播和组播 (Linux扩展)
    ///
    /// 允许广播和监听组播
    NetBroadcast = 11,

    /// 网络管理权限 (Linux扩展)
    ///
    /// 允许：
    /// - 接口配置
    /// - IP防火墙/伪装/统计管理
    /// - 设置套接字调试选项
    /// - 修改路由表
    /// - 设置套接字任意进程/进程组所有权
    /// - 透明代理绑定任意地址
    /// - 设置服务类型（TOS）
    /// - 设置混杂模式
    /// - 清除驱动统计信息
    /// - 组播
    NetAdmin = 12,

    /// 使用原始套接字 (Linux扩展)
    ///
    /// 允许使用 RAW 和 PACKET 套接字
    /// 允许透明代理绑定任意地址
    NetRaw = 13,

    /// 锁定内存页 (Linux扩展)
    ///
    /// 允许锁定共享内存段
    /// 允许 mlock 和 mlockall
    IpcLock = 14,

    /// 绕过IPC所有权检查 (Linux扩展)
    ///
    /// 覆盖IPC所有权检查
    IpcOwner = 15,

    /// 内核模块操作 (Linux扩展)
    ///
    /// 允许插入/移除内核模块（无限制修改内核）
    SysModule = 16,

    /// 直接硬件访问 (Linux扩展)
    ///
    /// 允许 ioperm/iopl 访问
    /// 允许通过 /dev/bus/usb 发送USB消息到任意设备
    SysRawio = 17,

    /// 使用 chroot(2) (Linux扩展)
    ///
    /// 允许使用 chroot()
    SysChroot = 18,

    /// 跟踪任意进程 (Linux扩展)
    ///
    /// 允许 ptrace() 任意进程
    SysPtrace = 19,

    /// 配置进程统计 (Linux扩展)
    ///
    /// 允许配置进程统计（process accounting）
    SysPacct = 20,

    /// 系统管理权限 (Linux扩展)
    ///
    /// 允许：
    /// - 配置安全注意键（SAK）
    /// - 管理随机设备
    /// - 配置磁盘配额
    /// - 设置域名
    /// - 设置主机名
    /// - mount()/umount()，新建SMB连接
    /// - 部分autofs根操作
    /// - nfsservctl
    /// - 虚拟机中断请求（VM86_REQUEST_IRQ）
    /// - Alpha架构PCI配置读写
    /// - MIPS架构irix_prctl（setstacksize）
    /// - m68k缓存刷新（sys_cacheflush）
    /// - 移除信号量
    /// - IPC对象所有权变更（替代`CAP_CHOWN`）
    /// - 锁定/解锁共享内存段
    /// - 启用/禁用交换空间
    SysAdmin = 21,

    /// 系统重启权限 (Linux扩展)
    ///
    /// 允许使用 reboot()
    SysBoot = 22,

    /// 提升优先级 (Linux扩展)
    ///
    /// 允许：
    /// - 提升优先级
    /// - 设置不同UID进程的优先级
    /// - 设置自身进程的FIFO/RR调度策略
    /// - 设置其他进程的调度策略
    /// - 设置其他进程的CPU亲和性
    /// - 设置实时I/O优先级类
    SysNice = 23,

    /// 覆盖资源限制 (Linux扩展)
    ///
    /// 允许：
    /// - 覆盖资源限制
    /// - 设置资源限制
    /// - 覆盖配额限制
    /// - 覆盖ext2保留空间
    /// - 修改ext3日志模式
    /// - 覆盖IPC消息队列大小限制
    /// - 允许实时时钟>64Hz中断
    /// - 覆盖控制台分配的最大数量
    /// - 覆盖键盘映射最大数量
    /// - 控制内存回收行为
    SysResource = 24,

    /// 系统时钟操作 (Linux扩展)
    ///
    /// 允许：
    /// - 操纵系统时钟
    /// - MIPS架构irix_stime
    /// - 设置实时时钟
    SysTime = 25,

    /// 终端设备配置 (Linux扩展)
    ///
    /// 允许配置TTY设备
    /// 允许对TTY使用 vhangup()
    SysTtyConfig = 26,

    /// 创建设备节点 (Linux扩展)
    ///
    /// 允许 mknod() 的特权操作
    Mknod = 27,

    /// 文件租约 (Linux扩展)
    ///
    /// 允许在文件上获取租约（leases）
    Lease = 28,

    /// 写入审计日志 (Linux扩展)
    ///
    /// 允许通过单播netlink套接字写入审计日志
    AuditWrite = 29,

    /// 配置审计系统 (Linux扩展)
    ///
    /// 允许通过单播netlink套接字配置审计
    AuditControl = 30,

    /// 文件能力操作 (Linux扩展)
    ///
    /// 允许在文件上设置/移除能力
    /// 允许将uid=0映射到子用户命名空间
    Setfcap = 31,

    /// 覆盖MAC访问控制 (Linux扩展)
    ///
    /// 基础内核不强制MAC策略。
    /// LSM模块可基于此能力实现策略覆盖
    MacOverride = 32,

    /// MAC策略管理 (Linux扩展)
    ///
    /// 基础内核不要求MAC配置。
    /// LSM模块可基于此能力实现策略修改检查
    MacAdmin = 33,

    /// 内核日志配置 (Linux扩展)
    ///
    /// 允许配置内核syslog（printk行为）
    Syslog = 34,

    /// 触发系统唤醒 (Linux扩展)
    ///
    /// 允许触发系统唤醒操作
    WakeAlarm = 35,

    /// 阻止系统挂起 (Linux扩展)
    ///
    /// 允许阻止系统挂起
    BlockSuspend = 36,

    /// 读取审计日志 (Linux扩展)
    ///
    /// 允许通过组播netlink套接字读取审计日志
    AuditRead = 37,

    /// 系统性能监控 (Linux扩展)
    ///
    /// 允许使用perf_events等子系统执行性能监控特权操作
    Perfmon = 38,

    /// BPF高级功能 (Linux扩展)
    ///
    /// 允许：
    /// - 创建所有类型BPF映射
    /// - 使用高级验证器功能（间接访问/循环等）
    /// - 加载BPF类型格式（BTF）
    /// - 检索BPF程序的JIT代码
    /// - 使用bpf_spin_lock()助手
    Bpf = 39,

    /// 检查点/恢复操作 (Linux扩展)
    ///
    /// 允许：
    /// - 检查点/恢复相关操作
    /// - clone3()期间选择PID
    /// - 写入ns_last_pid
    CheckpointRestore = 40,
}

impl Capability {
    /// 最后支持的能力值 (`CAP_LAST_CAP`)
    pub const LAST: Capability = Capability::CheckpointRestore;

    /// 检查能力值是否有效 (`cap_valid`)
    pub fn is_valid(value: u32) -> bool {
        value <= Self::LAST as u32
    }

    /// 计算能力在能力集中的索引 (`CAP_TO_INDEX`)
    ///
    /// 返回该能力在u32数组中的位置索引
    pub fn to_index(self) -> usize {
        (self as u32 as usize) >> 5
    }

    /// 计算能力在u32中的位掩码 (`CAP_TO_MASK`)
    ///
    /// 返回该能力在u32数组元素中的位掩码
    pub fn to_mask(self) -> u32 {
        1 << ((self as u32) & 31)
    }

    /// 从整数值创建能力枚举
    ///
    /// 如果值无效则返回None
    pub fn from_u32(value: u32) -> Option<Self> {
        if !Self::is_valid(value) {
            return None;
        }
        Some(unsafe { core::mem::transmute(value) })
    }
}

// 为枚举实现迭代功能
impl IntoIterator for Capability {
    type Item = Self;
    type IntoIter = core::iter::Copied<core::slice::Iter<'static, Self>>;

    fn into_iter(self) -> Self::IntoIter {
        ALL_CAPABILITIES.iter().copied()
    }
}

/// 所有能力值的静态数组
const ALL_CAPABILITIES: [Capability; 41] = [
    Capability::Chown,
    Capability::DacOverride,
    Capability::DacReadSearch,
    Capability::Fowner,
    Capability::Fsetid,
    Capability::Kill,
    Capability::Setgid,
    Capability::Setuid,
    Capability::Setpcap,
    Capability::LinuxImmutable,
    Capability::NetBindService,
    Capability::NetBroadcast,
    Capability::NetAdmin,
    Capability::NetRaw,
    Capability::IpcLock,
    Capability::IpcOwner,
    Capability::SysModule,
    Capability::SysRawio,
    Capability::SysChroot,
    Capability::SysPtrace,
    Capability::SysPacct,
    Capability::SysAdmin,
    Capability::SysBoot,
    Capability::SysNice,
    Capability::SysResource,
    Capability::SysTime,
    Capability::SysTtyConfig,
    Capability::Mknod,
    Capability::Lease,
    Capability::AuditWrite,
    Capability::AuditControl,
    Capability::Setfcap,
    Capability::MacOverride,
    Capability::MacAdmin,
    Capability::Syslog,
    Capability::WakeAlarm,
    Capability::BlockSuspend,
    Capability::AuditRead,
    Capability::Perfmon,
    Capability::Bpf,
    Capability::CheckpointRestore,
];