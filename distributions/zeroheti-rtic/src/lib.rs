#![no_std]

pub mod export;

pub use rtic_macro::app;
pub use rtic_observability::RticObservability;

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fintc-hetic, -Fintc-clic, -Fintc-edfic"
);
