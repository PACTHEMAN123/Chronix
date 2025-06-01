use alloc::{collections::btree_map::BTreeMap,sync::{Arc,Weak}, task, vec::Vec};
use async_task::Task;
use log::info;
use spin::Lazy;

use crate::{processor::processor::current_processor, sync::mutex::SpinNoIrqLock, syscall::process};

use super::{task::TaskControlBlock, tid::{PGid,Pid,Tid}, INITPROC, INITPROC_PID};
/// Task manager to manage all tasks in the system.
pub struct TaskManager (SpinNoIrqLock<BTreeMap<Tid, Arc<TaskControlBlock>>>);
impl TaskManager {
    /// Create a new `TaskManager`
    pub fn new() -> Self {
        TaskManager(SpinNoIrqLock::new(BTreeMap::new()))
    }
    /// add a task to the task manager
    pub fn add_task(&self, task: &Arc<TaskControlBlock>) {
        self.0.lock().insert(task.tid(), task.clone());
    }
    /// remove a task from the task manager
    pub fn remove_task(&self, tid: Tid) {
        self.0.lock().remove(&tid);
    }
    /// get the task by tid
    pub fn get_task(&self, tid: Tid) -> Option<Arc<TaskControlBlock>> {
        match self.0.lock().get(&tid) {
            Some(task)  => Some(task.clone()),
            None => None,
        }
    }
    /// get the init task
    pub fn get_init_proc(&self) -> Arc<TaskControlBlock> {
        self.get_task(INITPROC_PID).unwrap()
    }
    /// turn BtreeMap into a vector of tasks
    pub fn tasks_group(&self) -> Vec<Arc<TaskControlBlock>> {
        self.0.lock()
        .values()
        .map(|task| task.clone())
        .collect()
    }
    /// do something for each task
    pub fn for_each_task<F: FnMut(&Arc<TaskControlBlock>)>(&self, mut f: F) {
        for task in self.tasks_group() {
            f(&task)
        }
    }
}
/// Group of tasks with the same process group id doing some work together.
pub struct ProcessGroupManager (SpinNoIrqLock<BTreeMap<PGid, Vec<Weak<TaskControlBlock>>>>);

impl ProcessGroupManager {
    /// Create a new `ProcessGroupManager`
    pub const fn new() -> Self {
        Self(SpinNoIrqLock::new(BTreeMap::new()))
    }
    /// initiate a group by group leader
    pub fn add_group(&self, group_leader: &Arc<TaskControlBlock>) {
        info!("add group with leader {}, now pgid is {}" , group_leader.tid(), group_leader.pgid());
        let pgid = group_leader.tid();
        group_leader.set_pgid(pgid);
        info!("add group with leader {}, now pgid is {}" , group_leader.tid(), group_leader.pgid());
        let mut group = Vec::new();
        group.push(Arc::downgrade(group_leader));
        self.0.lock().insert(pgid, group); 
        info!("insert group {} with leader {}" , pgid, group_leader.tid());
    }
    /// add a task to a group
    pub fn add_task_to_group(&self,pgid: PGid, task: &Arc<TaskControlBlock>) {
        //info!("add task {} to group {}, processor id {}", task.tid(), pgid, current_processor().id() );
        task.set_pgid(pgid);
        self.0.lock().get_mut(&pgid).unwrap()
        .push(Arc::downgrade(task));
    }
    /// get a group by pgid
    pub fn get_group(&self, pgid: PGid) -> Option<Vec<Weak<TaskControlBlock>>> {
        self.0.lock().get(&pgid).cloned()
    }
    /// remove a task from a group
    pub fn remove(&self, task: &Arc<TaskControlBlock>) {
        //info!("remove task {} from group {}", task.tid(), task.pgid());
        self.0.lock().get_mut(&task.pgid()).unwrap()
        .retain(|t|t.upgrade().map_or(false, |inner| Arc::ptr_eq(task, &inner)));
    }
}
/// The global task manager
pub static TASK_MANAGER: Lazy<TaskManager> = Lazy::new(TaskManager::new);
/// The global process group manager
pub static PROCESS_GROUP_MANAGER: ProcessGroupManager = ProcessGroupManager::new();