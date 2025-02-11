use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use rtic_core::{AppArgs, CorePassBackend, RticMacroBuilder, SubAnalysis, SubApp};
use syn::{parse_quote, ItemFn};

extern crate proc_macro;

struct AtalantaRtic;

use rtic_sw_pass::SoftwarePass;

const MIN_TASK_PRIORITY: u16 = 1; // lowest Atalanta priority

#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    // use the standard software pass provided by rtic-sw-pass crate
    let sw_pass = SoftwarePass::new(SwPassBackend);

    let mut builder = RticMacroBuilder::new(AtalantaRtic);
    builder.bind_pre_core_pass(sw_pass);
    builder.build_rtic_macro(args, input)
}

// =========================================== Trait implementations ===================================================
impl CorePassBackend for AtalantaRtic {
    fn default_task_priority(&self) -> u16 {
        MIN_TASK_PRIORITY
    }

    fn post_init(
        &self,
        _app_args: &AppArgs,
        _sub_app: &SubApp,
        app_analysis: &SubAnalysis,
    ) -> Option<TokenStream2> {
        let initialize_dispatcher_interrupts =
            app_analysis.used_irqs.iter().map(|(irq_name, priority)| {
                let priority = priority.max(&MIN_TASK_PRIORITY); // limit piority to minmum
                quote! {
                    //set interrupt priority
                    rtic::export::enable(
                        rtic::export::interrupts::#irq_name,
                        #priority as u8,
                    );
                }
            });

        Some(quote! {
            // Use 8 bits for level. Omitting this will cause the hardware to not respect interrupt level.
            bsp::clic::Clic::smclicconfig().set_mnlbits(8);

            unsafe {
                #(#initialize_dispatcher_interrupts)*
            }
        })
    }

    fn populate_idle_loop(&self) -> Option<TokenStream2> {
        Some(quote! {
            unsafe { core::arch::asm!("wfi") };
        })
    }

    fn generate_interrupt_free_fn(&self, mut empty_body_fn: ItemFn) -> ItemFn {
        // eprintln!("{}", empty_body_fn.to_token_stream().to_string()); // enable comment to see the function signature
        let fn_body = parse_quote! {
            {
                rtic::export::interrupt_disable();
                let r = f();
                unsafe { rtic::export::interrupt_enable(); } // critical section end
                r
            }
        };
        empty_body_fn.block = Box::new(fn_body);
        empty_body_fn
    }

    fn generate_global_definitions(
        &self,
        _app_args: &AppArgs,
        _app_info: &SubApp,
        _app_analysis: &SubAnalysis,
    ) -> Option<TokenStream2> {
        None
    }

    fn generate_resource_proxy_lock_impl(
        &self,
        _app_args: &AppArgs,
        _app_info: &SubApp,
        incomplete_lock_fn: syn::ImplItemFn,
    ) -> syn::ImplItemFn {
        let lock_impl: syn::Block = parse_quote! {
            {
                unsafe { rtic::export::lock(resource_ptr, task_priority as u8, CEILING as u8, f); }
            }
        };

        let mut completed_lock_fn = incomplete_lock_fn;
        completed_lock_fn.block.stmts.extend(lock_impl.stmts);
        completed_lock_fn
    }

    fn entry_name(&self, _core: u32) -> Ident {
        // same entry name for both cores.
        // two main() functions will be generated but both will be guarded by #[cfg(core = "X")]
        // each generated binary will have have one entry
        format_ident!("main")
    }

    /// Customize how the task is dispatched when its bound interrupt is triggered (save basepri
    /// before and restore after executing the task)
    fn wrap_task_execution(
        &self,
        _task_prio: u16,
        _dispatch_task_call: TokenStream2,
    ) -> Option<TokenStream2> {
        None
    }

    /// further analysis of parsed user code
    fn pre_codegen_validation(
        &self,
        _app: &rtic_core::App,
        _analysis: &rtic_core::Analysis,
    ) -> syn::Result<()> {
        Ok(())
    }

    fn entry_attrs(&self) -> Vec<syn::Attribute> {
        vec![syn::parse_quote!(#[bsp::rt::entry])]
    }

    fn task_attrs(&self) -> Vec<syn::Attribute> {
        vec![syn::parse_quote!(#[bsp::nested_interrupt])]
    }
}

struct SwPassBackend;
impl rtic_sw_pass::SwPassBackend for SwPassBackend {
    /// Provide the implementation/body of the core local interrupt pending function.
    fn generate_local_pend_fn(&self, mut empty_body_fn: ItemFn) -> ItemFn {
        let body = parse_quote!({
            rtic::export::pend(irq_nbr);
        });
        empty_body_fn.block = Box::new(body);
        empty_body_fn
    }

    /// Provide the implementation/body of the cross-core interrupt pending function.
    fn generate_cross_pend_fn(&self, _empty_body_fn: ItemFn) -> Option<ItemFn> {
        None
    }

    /// Provide a custom path for interrupts list
    fn custom_interrupt_path(&self, _core: u32) -> Option<syn::Path> {
        Some(parse_quote!(rtic::export::interrupts))
    }
}
