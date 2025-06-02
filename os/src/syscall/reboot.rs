use core::time::Duration;

use crate::{executor::os_send_shutdown, signal::{SigInfo, SIGTERM}, task::{current_task, manager::TASK_MANAGER, INITPROC_PID}, timer::timed_task::suspend_timeout};

use super::SysError;

pub async fn sys_reboot(_magic1: i32, _magic2: i32, _cmd: u32, _arg: usize) -> Result<isize, SysError> {
    TASK_MANAGER.for_each_task(|task| {
        if task.tid() == INITPROC_PID {
            return;
        }
        task.recv_sigs(SigInfo { si_signo: SIGTERM, si_code: SigInfo::KERNEL, si_pid: None });
    });
    // wait 0.5s
    suspend_timeout(current_task().unwrap(), Duration::from_millis(500)).await;
    os_send_shutdown();
    Ok(0)
}