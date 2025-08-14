use alloc::string::ToString;
use hal::timer::{Timer, TimerHal};

use crate::fs::tmpfs::inode::InodeContent;


pub struct CpuInfo;

impl CpuInfo {
    pub const fn new() -> Self {
        Self {}
    }
}

impl InodeContent for CpuInfo {
    fn serialize(&self) -> alloc::string::String {
        let cpu_freq_mhz = Timer::get_timer_freq() / 1_000_000;
        let mut res = "".to_string();
        res += &"processor\t: 0\n".to_string();
        res += &"vendor_id\t: Intel\n".to_string();
        res += &"cpu family\t: 5\n".to_string();
        res += &"model\t: 44\n".to_string();
        res += &"model name\t: Intel Sucks\n".to_string();
        res += &"stepping\t: 2\n".to_string();

        res += &"MHz\t: ".to_string();
        res += &cpu_freq_mhz.to_string();
        res += &"\n".to_string();

        res += &"cache size\t: 512 KB\n".to_string();
        res += &"physical id\t: 0\n".to_string();
        res += &"siblings\t: 1\n".to_string();
        res += &"runqueue\t: 0\n".to_string();
        res += &"fdiv_bug\t: no\n".to_string();
        res += &"hlt_bug\t: no\n".to_string();
        res += &"f00f_bug\t: no\n".to_string();
        res += &"coma_bug\t: no\n".to_string();
        res += &"fpu\t: yes\n".to_string();
        res += &"fpu_exception\t: yes\n".to_string();
        res += &"cpuid level\t: 2\n".to_string();
        res += &"wp\t: yes\n".to_string();
        res += &"flags\t: fpu vme de pse tsc msr pae mce\n".to_string();
        res
    }
}