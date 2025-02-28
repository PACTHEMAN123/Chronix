use crate::{
    timer::{ffi::TimeVal, get_current_time_ms},
    mm::UserCheck,
};

/// get current time of day
pub fn sys_gettimeofday(tv: *mut TimeVal) -> isize {
    let user_check = UserCheck::new();
    user_check.check_write_slice(tv as *mut u8, core::mem::size_of::<TimeVal>());

    let current_time = get_current_time_ms();
    let time_val = TimeVal {
        sec: current_time / 1000,
        usec: (current_time % 1000) * 1000,
    };
    
    unsafe {
        tv.write_volatile(time_val);
    }
    0
}