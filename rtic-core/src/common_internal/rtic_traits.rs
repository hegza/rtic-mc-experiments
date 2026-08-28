use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

pub const HWT_TRAIT_TY: &str = "RticTask";
pub const SWT_TRAIT_TY: &str = "RticSwTask"; // FIXME: add a backend trait method to provide a list of additional traits that define task types instead of this wrong way of borrowing from sw pass implicitly !
pub const IDLE_TRAIT_TY: &str = "RticIdleTask";

pub const MUTEX_TY: &str = "RticMutex";
pub const READ_LOCK_TY: &str = "RticReadLock";

/// Emit the generated `rtic_traits` module into the app module.
///
pub(crate) fn get_rtic_traits_mod(obs: Option<&syn::Path>) -> TokenStream2 {
    let hw_task_trait = hw_task_trait();
    let idle_trait = idle_task_trait();
    let mutex_trait = mutex_trait();
    let read_lock_trait = read_lock_trait();
    let obs_trait = obs.map(|_| quote! { pub use ::rtic::RticObservability; });
    quote! {
        /// Module defining rtic traits
        pub use rtic_traits::*;
        pub mod rtic_traits {
            #hw_task_trait
            #idle_trait
            #mutex_trait
            #read_lock_trait
            #obs_trait
        }
    }
}

fn hw_task_trait() -> TokenStream2 {
    let hw_task = format_ident!("{HWT_TRAIT_TY}");
    quote! {
        /// Trait for a hardware task
        pub trait #hw_task {
            /// Associated type that can be used to make [Self::init] take arguments
            type InitArgs: Sized;
            /// Task local variables initialization routine
            fn init(args: Self::InitArgs) -> Self;
            /// Function to be bound to a HW Interrupt
            fn exec(&mut self);
        }
    }
}

fn idle_task_trait() -> TokenStream2 {
    let idle_task = format_ident!("{IDLE_TRAIT_TY}");
    quote! {
        /// Trait for an idle task
        pub trait #idle_task {
            /// Associated type that can be used to make [Self::init] take arguments
            type InitArgs: Sized;
            /// Task local variables initialization routine
            fn init(args: Self::InitArgs) -> Self;
            /// Function to be executing when no other task is running
            fn exec(&mut self) -> !;
        }
    }
}
fn mutex_trait() -> TokenStream2 {
    let mutex = format_ident!("{MUTEX_TY}");
    quote! {
        pub trait #mutex {
            type ResourceType;
            fn lock<R>(&mut self, f: impl FnOnce(&mut Self::ResourceType) -> R) -> R;
        }
    }
}
fn read_lock_trait() -> TokenStream2 {
    let read_lock = format_ident!("{READ_LOCK_TY}");
    quote! {
        pub trait #read_lock {
            type ResourceType;
            fn read_lock<R>(&self, f: impl FnOnce(&Self::ResourceType) -> R) -> R;
        }
    }
}
