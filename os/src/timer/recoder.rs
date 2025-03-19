use core::time::Duration;

use super::get_current_time_duration;
/// Time recoder for events in tasks and kernel functions
/// Todo:need to distinguish time for calculating cpu usage, time for IO or NET, time for sleeping, etc.
pub struct TimeRecorder {
    /// user time duration
    user_time: Duration,
    /// kernel time duration
    kernel_time: Duration,
    /// kernel time start
    kernel_start: Duration,
    /// user time start
    user_start: Duration,
    /// for a parent task need to record the child task's time
    /// child user time Duration
    child_user_time: Duration,
    /// child kernel time Duration
    child_kernel_time: Duration,
}

impl TimeRecorder {
    /// new cosnt TimeRecorder
    pub const fn new() -> Self {
        Self {
            user_time: Duration::ZERO,
            kernel_time: Duration::ZERO,
            kernel_start: Duration::ZERO,
            user_start: Duration::ZERO,
            child_user_time: Duration::ZERO,
            child_kernel_time: Duration::ZERO,
        }
    }
    /// return a pair for user and kernel time
    pub fn time_pair(&self) -> (Duration, Duration) {
        (self.user_time, self.kernel_time)
    }
    /// return a pair for child user and kernel time
    pub fn child_time_pair(&self) -> (Duration, Duration) {
        (self.child_user_time, self.child_kernel_time)
    }
    
    #[inline]
    /// user time method
    pub fn user_time(&self) -> Duration {
        self.user_time
    }
    #[inline]
    /// kernel time method
    pub fn kernel_time(&self) -> Duration {
        self.kernel_time
    }
    /// time for cacluating cpu usage
    pub fn processor_time(&self) -> Duration {
        self.kernel_time + self.user_time
    }
    /// update user time start
    pub fn update_user_start(&mut self, user_start: Duration) {
        self.user_start = user_start;
    }
    /// for parent task to update child task's time
    pub fn update_child_time(&mut self, (child_user_time, child_kernel_time): (Duration, Duration)) {
        self.child_user_time += child_user_time;
        self.child_kernel_time += child_kernel_time;
    }
    /// for switch_to_current_task recording 
    pub fn record_switch_in(&mut self) {
        let current_time = get_current_time_duration();
        self.kernel_start = current_time;
    }
    /// for switch_out_current_task recording
    pub fn record_switch_out(&mut self) {
        let current_time = get_current_time_duration();
        let kernel_time_slice = current_time - self.kernel_start;
        self.kernel_time += kernel_time_slice;
    }
    /// for trap recording: from user to kernel
    pub fn record_trap(&mut self){
        let current_time = get_current_time_duration();
        self.kernel_start = current_time;
        self.user_time += current_time - self.user_start;
    }
    /// for trap_return recording: form kernel to user
    pub fn record_trap_return(&mut self){
        let current_time = get_current_time_duration();
        self.kernel_time += current_time - self.user_start;    
        self.user_start = current_time;
    }
}