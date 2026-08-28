use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ImplItemFn, ItemFn, parse_quote};

use crate::{
    Analysis, AppArgs, CorePassBackend, SubApp, multibin,
    parser::ast::{RticTask, SharedElement},
};

pub const INTERRUPT_FREE_FN: &str = "__rtic_interrupt_free";

pub(crate) fn get_interrupt_free_fn(implementor: &dyn CorePassBackend) -> ItemFn {
    let fn_ident = format_ident!("{INTERRUPT_FREE_FN}");
    let critical_section_fn = parse_quote! {
        #[inline]
        pub fn #fn_ident<F, R>(f: F) -> R
        where F: FnOnce() -> R,
        {
           // IMPLEMENTOR RESPONSIBILITY: implement a traditional interrupt critical section
        }
    };
    implementor.generate_interrupt_free_fn(critical_section_fn)
    // TODO: we should validate if the implementor has kept the correct function signature by comparing it to the initial signature
}

pub(crate) fn get_resource_proxy_lock_fn(
    implementor: &dyn CorePassBackend,
    app_params: &AppArgs,
    app_info: &SubApp,
    resource: &SharedElement,
    static_mut_shared_resources: &syn::Ident,
) -> ImplItemFn {
    let ceiling = resource.priority;
    let resource_ident = &resource.ident;
    let lock_fn: syn::ImplItemFn = parse_quote! {
        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::ResourceType) -> R) -> R {
            // `self` refers to the resource proxy struct

            const CEILING: u16 = #ceiling; // resource priority ceiling
            let task_priority = self.task_priority; // running task priority
            let resource_ptr = unsafe { // get a mut pointer to the resource
                &mut #static_mut_shared_resources.assume_init_mut().#resource_ident
            } as *mut _;
            // IMPLEMENTOR RESPONSIBILITY: continue lock implementation here
            // call for example rtic::export::lock(resource_ptr, task_priority, ...., f)
        }
    };
    let preamble_len = lock_fn.block.stmts.len();
    let lock_fn = implementor.generate_resource_proxy_lock_impl(app_params, app_info, lock_fn);
    if app_params.obs.is_some() {
        return wrap_lock_fn_with_obs(lock_fn, resource, preamble_len);
    }
    lock_fn
    // TODO: we should validate if the implementor has kept the correct function signature by comparing it to the initial signature
}

pub(crate) fn get_resource_proxy_read_lock_fn(
    implementor: &dyn CorePassBackend,
    app_params: &AppArgs,
    app_info: &SubApp,
    resource: &SharedElement,
    static_mut_shared_resources: &syn::Ident,
) -> ImplItemFn {
    let ceiling = resource.read_priority;
    let resource_ident = &resource.ident;
    let lock_fn: syn::ImplItemFn = parse_quote! {
        fn read_lock<R>(&self, f: impl FnOnce(&Self::ResourceType) -> R) -> R {
            // `self` refers to the resource proxy struct

            const CEILING: u16 = #ceiling; // resource priority ceiling
            let task_priority = self.task_priority; // running task priority
            let resource_ptr = unsafe { // get a mut pointer to the resource
                &mut #static_mut_shared_resources.assume_init_mut().#resource_ident
            } as *mut _;
            let f = |resource: &mut Self::ResourceType| {
                f(resource)
            };
            // IMPLEMENTOR RESPONSIBILITY: continue lock implementation here
            // call for example rtic::export::lock(resource_ptr, task_priority, ...., f)
        }
    };
    let preamble_len = lock_fn.block.stmts.len();
    let lock_fn = implementor.generate_resource_proxy_lock_impl(app_params, app_info, lock_fn);
    if app_params.obs.is_some() {
        return wrap_lock_fn_with_obs(lock_fn, resource, preamble_len);
    }
    lock_fn
    // TODO: we should validate if the implementor has kept the correct function signature by comparing it to the initial signature
}

/// Wrap the (backend-completed) lock/read_lock function body with the
/// observability hooks `on_res_acq`/`on_res_rel`.
///
/// The first `preamble_len` statements of the body are the rtic-core generated
/// preamble (`const CEILING`, `let task_priority`, `let resource_ptr` and, for
/// `read_lock`, `let f`); the backend only appends statements to that skeleton,
/// so the remaining statements form the actual critical-section implementation
/// which is captured into `__rtic_obs_result`. The return value `R` is
/// preserved exactly.
fn wrap_lock_fn_with_obs(
    mut lock_fn: ImplItemFn,
    resource: &SharedElement,
    preamble_len: usize,
) -> ImplItemFn {
    let res_variant = format_ident!("{}", resource.ident.to_string().to_upper_camel_case());
    let mut block = lock_fn.block;
    let mut stmts = block.stmts;
    let backend_stmts: Vec<_> = stmts.split_off(preamble_len);

    let mut new_stmts = stmts;
    new_stmts.push(parse_quote! {
        <__rtic_obs as RticObservability>::on_res_acq(ResourceId::#res_variant as u8, task_priority, CEILING);
    });
    new_stmts.push(parse_quote! {
        let __rtic_obs_result = { #(#backend_stmts)* };
    });
    new_stmts.push(parse_quote! {
        <__rtic_obs as RticObservability>::on_res_rel(ResourceId::#res_variant as u8, task_priority, CEILING);
    });
    // unwritten as a `Stmt` (which requires a trailing `;`); keep it as a
    // semicolon-free expression statement so the captured `R` value flows out.
    let result_expr: syn::Expr = parse_quote! {
        __rtic_obs_result
    };
    new_stmts.push(syn::Stmt::Expr(result_expr, None));
    block.stmts = new_stmts;

    lock_fn.block = block;
    lock_fn
}

pub(crate) fn task_trait_check_fn_name(trait_ident: &syn::Ident) -> syn::Ident {
    let trait_lower = trait_ident.to_string().to_snake_case();
    format_ident!("implements_{trait_lower}")
}
pub(crate) fn trait_check_call_for(task: &RticTask) -> TokenStream {
    let task_trait = &task.args.task_trait;
    let task_ty = &task.task_struct.ident;
    let check_fn_name = task_trait_check_fn_name(task_trait);
    let core = task.args.core;
    let cfg_core = multibin::multibin_cfg_core(core);

    quote! {
        #cfg_core
        const _: fn() = || {
            __rtic_trait_checks::#check_fn_name::<#task_ty>();
        };
    }
}

fn generate_trait_check_fn(task_trait: &syn::Ident) -> TokenStream {
    let check_fn_name = task_trait_check_fn_name(task_trait);
    quote! {
        pub fn #check_fn_name<T: #task_trait>(){}
    }
}

pub(crate) fn generate_task_traits_check_functions(analysis: &Analysis) -> TokenStream {
    let function_definitions = analysis.task_traits.iter().map(generate_trait_check_fn);
    quote! {
        mod __rtic_trait_checks {
            use super::*;
            #(#function_definitions)*
        }
    }
}
