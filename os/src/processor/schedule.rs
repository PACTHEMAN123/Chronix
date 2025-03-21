use core::sync::atomic::{AtomicBool, AtomicUsize};
use alloc::sync::Arc;
use alloc::task;
use async_task::Schedule;
use log::info;
use crate::sync::mutex::SpinNoIrqLock;
use crate::task::task::TaskControlBlock;
use crate::task::Processor;
use crate::timer::{get_current_time, get_current_time_us};
use lazy_static::lazy_static;
use crate::board::MAX_PROCESSORS;
use crate::processor::processor::PROCESSORS;
use alloc::vec::Vec;
#[cfg(feature = "smp")]
const LOAD_THRESHOLD: usize = 10;
#[cfg(feature = "smp")]
/// plet algorithm: Partial reference to linux
/// a float number will multiply 2^32 , represented as a integer number which higher 10 bits stands for integer part and lower 22 bits stands for decimal part
/// y^0 - y^31
static DECLINE_TABLE: [u32;32] = [
    0xffffffff, 0xfa83b2da, 0xf5257d14, 0xefe4b99a, 0xeac0c6e6, 0xe5b906e6,
	0xe0ccdeeb, 0xdbfbb796, 0xd744fcc9, 0xd2a81d91, 0xce248c14, 0xc9b9bd85,
	0xc5672a10, 0xc12c4cc9, 0xbd08a39e, 0xb8fbaf46, 0xb504f333, 0xb123f581,
	0xad583ee9, 0xa9a15ab4, 0xa5fed6a9, 0xa2704302, 0x9ef5325f, 0x9b8d39b9,
	0x9837f050, 0x94f4efa8, 0x91c3d373, 0x8ea4398a, 0x8b95c1e3, 0x88980e80,
	0x85aac367, 0x82cd8698,
];
#[cfg(feature = "smp")]
const LOAD_AVG_PERIOD: u32 = 32;
#[cfg(feature = "smp")]
/// the sum of Geometric sequence of the decay factor 
const LOAD_AVG_MAX: u32 = 47742;
#[cfg(feature = "smp")]
const PELT_MIN_DIVIDER: u32 = LOAD_AVG_MAX - 1024;
#[cfg(feature = "smp")]
pub struct TaskLoadTracker {
    // last_update_time: micro_second, used to caculate delta between two updates
    pub last_update_time: u64,
    // load_sum of the task to the run_queue, including both waiting time and running time
    pub load_sum: u64,
    /// note the unimplemented time windows when last update ends
    pub period_contribute: u32,
    /// avg_load
    pub load_avg: u32,
}
#[cfg(feature = "smp")]
impl TaskLoadTracker {
    pub const fn new() -> Self {
        Self {
            last_update_time: 0,
            load_sum: 0,
            period_contribute: 0,
            load_avg: 0,
        }
    }
    pub fn last_update_time_us(&self) -> u64 {
        self.last_update_time
    }
    pub fn get_pelt_divider(&self) -> u32 {
        self.period_contribute + PELT_MIN_DIVIDER
    }
}
#[cfg(feature = "smp")]
/// return val*y^n
fn decay_load(mut val: u64, num: u64) -> u64 {
    if num > LOAD_AVG_PERIOD as u64 * 63 {
        return 0;    
    }
    let mut inner_num = num as u32;

    //y^n = 1/2^(n/PERIOD) * y^(n%PERIOD)
    if inner_num >= LOAD_AVG_PERIOD {
        let shift = inner_num / LOAD_AVG_PERIOD;
        val >>= shift;
        inner_num %= LOAD_AVG_PERIOD;
    }

    let factor = DECLINE_TABLE[inner_num as usize];
    val = ((val as u128 * factor as u128) >> 32) as u64;
    val
}
#[cfg(feature = "smp")]
fn _add_segment(period_num: u64, load1: u32, load3: u32) ->  u32 {
    let c1 = decay_load(load1 as u64, period_num) ;
    let c2 = LOAD_AVG_MAX as u64 - decay_load(LOAD_AVG_MAX as u64, period_num) - 1024;
    let c3 = load3 as u64;
    (c1 + c2 + c3) as u32
}
#[cfg(feature = "smp")]
impl TaskLoadTracker {
    pub fn update_task_load(&mut self, mut delta: u64) -> u32 {
        let mut contribute = 0;
        delta += (self.period_contribute) as u64;
        let periods = delta / 1024;    // get periods nums
        // decay old sum if we crossed period boundaries
        if periods != 0 {
            self.load_sum = decay_load(self.load_sum, periods);

            delta %= 1024;
        
            // decay complete cycle
            contribute = _add_segment(periods, 1024 - self.period_contribute, delta as u32);
        }
        self.period_contribute = delta as u32;

        self.load_sum += contribute as u64; 
        // what is truly run by cpu
        return periods as u32;
    }
    /// despite some special cases, call update_task_load
    pub fn update_load_sum(&mut self, current: u64) -> u32 {
        let delta_time: u64 = current.wrapping_sub(self.last_update_time);
        if delta_time == 0 {
            return 0;
        }
        self.last_update_time = self.last_update_time.wrapping_add(delta_time) ;
        if self.update_task_load(delta_time) == 0 {
            return 0;
        }
        return 1;
    }
    /// update load_avg and running_avg
    pub fn _update_load_avg(&mut self) {
        let divider = self.get_pelt_divider();
        self.load_avg = (self.load_sum  / (divider as u64)) as u32;
    }
    /// for a sche entity, update load_avg and running_avg
    pub fn update_load_avg_se(&mut self,now: u64) -> u32{
        if self.update_load_sum(now) != 0 {
            self._update_load_avg();
            return 1;
        }
        return 0;
    }
}
#[cfg(feature = "smp")]
impl Processor {
    /**
     * will be used to update load_avg and running_avg in the following cases:
     * 1. push a task to run_queue
     * 2. pop a task from run_queue
     * 3. scheduler tick, update task load
     */ 
    pub fn update_load_avg(&mut self) {
        let current_cputime_us = self.rq_task_clock();
        let current = self.current().unwrap();
        current.with_mut_sche_entity(|se| se.update_load_avg_se(current_cputime_us));
        if self.unwrap_with_mut_sche_entity(|se| se.update_load_sum(current_cputime_us)) != 0 {
            self.unwrap_with_mut_sche_entity(|se| se._update_load_avg());
        }
    }
}



#[cfg(feature = "smp")]
#[allow(unused)]
pub fn load_balance() -> bool {
    use core::sync::atomic::Ordering;
    use log::info;
    let mut loads = Vec::new();
    for i in 0..MAX_PROCESSORS {
        loads.push((i,unsafe { PROCESSORS[i].task_nums().load(Ordering::SeqCst) }));
    }
    let (busiest_core, busiest_load) = loads.iter().max_by_key(|(_, l)| l).unwrap();
    let (idlest_core, idlest_load) = loads.iter().min_by_key(|(_, l)| l).unwrap();
    info!("busiest core: {}, busiest load: {}, idlest core: {}, idlest load: {}", busiest_core, busiest_load, idlest_core, idlest_load);
    if *busiest_load - *idlest_load > LOAD_THRESHOLD {
        info!("over threshold,migrate tasks");
        let tasks_num_to_move = (*busiest_load - *idlest_load) / 2;
        migrate_tasks(*busiest_core, *idlest_core);
        return false;
    }
    else {
        return true;
    }
}
#[cfg(feature = "smp")]
#[allow(unused)]
fn migrate_tasks(from_core: usize, to_core: usize) {
    let task = unsafe{PROCESSORS[from_core].unwrap_with_mut_task_queue(|queue|queue.pop_back().unwrap())};
    unsafe{PROCESSORS[to_core].unwrap_with_mut_task_queue(|queue| queue.push_back(task))};
}
#[cfg(feature = "smp")]
pub fn select_run_queue_index() -> usize {
    use core::sync::atomic::{AtomicUsize, Ordering};

    use log::info;

    use super::processor::current_processor;
    static TASK_QUEUE_INDEX: AtomicUsize = AtomicUsize::new(2);
    //info!("lazy_static TASK_QUEUE_INDEX: {}", TASK_QUEUE_INDEX.load(Ordering::SeqCst));
    loop {
        let index = TASK_QUEUE_INDEX.fetch_add(1, Ordering::SeqCst) % (MAX_PROCESSORS) ;
        return index 
    }
} 


