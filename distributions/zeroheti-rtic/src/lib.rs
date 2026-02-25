#![no_std]

pub mod export;

pub use rtic_macro::app;

#[cfg(not(any(feature = "intc-hetic", feature = "intc-clic", feature = "intc-edfic")))]
compile_error!(
    "at least one interrupt controller feature is required, pass -Fintc-hetic, -Fintc-clic, -Fintc-edfic"
);
