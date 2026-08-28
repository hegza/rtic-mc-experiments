#![no_std]

/// Optional, zero-cost-when-disabled observability hooks for RTIC.
pub trait RticObservability {
    /// Task activation: the task's job has started executing.
    #[inline(always)]
    fn on_task_act(_task_id: u8, _task_prio: u16) {}

    /// Task completion: the task's job has finished executing.
    #[inline(always)]
    fn on_task_comp(_task_id: u8, _task_prio: u16) {}

    /// Resource acquire: entering the SRP critical section.
    #[inline(always)]
    fn on_res_acq(_resource_id: u8, _task_prio: u16, _ceiling: u16) {}

    /// Resource release: leaving the SRP critical section.
    #[inline(always)]
    fn on_res_rel(_resource_id: u8, _task_prio: u16, _ceiling: u16) {}
}
