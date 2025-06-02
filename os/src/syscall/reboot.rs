use core::time::Duration;

use hal::println;

use crate::{executor::os_send_shutdown, signal::{SigInfo, SIGTERM}, task::{current_task, manager::TASK_MANAGER, INITPROC_PID}, timer::timed_task::suspend_timeout};

use super::SysError;

pub async fn sys_reboot(_magic1: i32, _magic2: i32, _cmd: u32, _arg: usize) -> Result<isize, SysError> {
    // let task = current_task().unwrap();
    // log::info!("[sys_reboot] task {} send reboot", task.tid());
    os_send_shutdown();
    Ok(0)
}