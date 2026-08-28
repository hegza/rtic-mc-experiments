#![no_std]

pub mod export;

pub use rtic_macro::app;

/// Optional, zero-cost-when-disabled observability hooks.
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

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fintc-hetic, -Fintc-clic, -Fintc-edfic"
);
