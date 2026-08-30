#![no_std]

/// Optional, zero-cost-when-disabled observability hooks for RTIC.
pub trait RticObservability {
    /// Task activation: the task's job has started executing.
    fn on_task_act(task_id: u8, task_prio: u16);

    /// Task completion: the task's job has finished executing.
    fn on_task_comp(task_id: u8, task_prio: u16);

    /// Resource acquire: entering the SRP critical section.
    ///
    /// # Arguments
    ///
    /// * `resource_id` - RTIC internal resource identifier
    /// * `task_prio` - priority of the locking task
    /// * `ceiling` - the new ceiling
    fn on_res_acq(resource_id: u8, task_prio: u16, ceiling: u16);

    /// Resource release: leaving the SRP critical section.
    ///
    /// # Arguments
    ///
    /// * `resource_id` - RTIC internal resource identifier
    /// * `task_prio` - priority of the locking task
    /// * `ceiling` - the restored ceiling
    fn on_res_rel(resource_id: u8, task_prio: u16, ceiling: u16);
}
