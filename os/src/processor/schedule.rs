use core::sync::atomic::{AtomicBool, AtomicUsize};
use async_task::Schedule;
use crate::sync::mutex::SpinNoIrqLock;
use lazy_static::lazy_static;
use crate::board::MAX_PROCESSORS;
use crate::processor::processor::PROCESSORS;
use alloc::vec::Vec;
#[cfg(feature = "smp")]
const LOAD_THRESHOLD: usize = 10;
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


