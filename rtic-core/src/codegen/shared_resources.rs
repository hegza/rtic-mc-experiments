use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::common::rtic_traits::MUTEX_TY;
use crate::multibin;
use crate::parser::ast::{RticTask, SharedResources};

impl SharedResources {
    pub fn generate_shared_resources_def(&self) -> TokenStream2 {
        let cfg_core = multibin::multibin_cfg_core(self.args.core);
        let shared_struct = &self.strct;
        let resources_ty = &shared_struct.ident;
        let static_instance_name = &self.name_uppercase();

        quote! {
            #cfg_core
            static mut #static_instance_name: core::mem::MaybeUninit<#resources_ty> = core::mem::MaybeUninit::uninit();
            #cfg_core
            #shared_struct
        }
    }

    pub fn generate_resource_proxies(&self, lock_implementation: &TokenStream2) -> TokenStream2 {
        let global_resources_handle = self.name_uppercase();
        let proxies = self.resources.iter().map(|element| {
            let element_name = &element.ident;
            let element_ty = &element.ty;
            let ceiling = &element.priority;
            let proxy_name = utils::get_proxy_name(element_name);
            let mutex_ty = format_ident!("{}", MUTEX_TY);
            let cfg_core = multibin::multibin_cfg_core(self.args.core);
            quote! {
                // Resource proxy for `#element_name`
                #cfg_core
                struct #proxy_name {
                    #[doc(hidden)]
                    priority: u16,
                }

                #cfg_core
                impl #proxy_name {
                    #[inline(always)]
                    pub fn new(priority: u16) -> Self {
                        Self { priority }
                    }
                }

                #cfg_core
                impl #mutex_ty for #proxy_name {
                    type ResourceType = #element_ty;
                    fn lock(&mut self, f: impl FnOnce(&mut #element_ty)) {
                        const CEILING: u16 = #ceiling;
                        let task_priority = self.priority;
                        let resource = unsafe {&mut #global_resources_handle.assume_init_mut().#element_name} as *mut _;
                        #lock_implementation
                    }
                }
            }
        });
        quote! {
            #(#proxies)*
        }
    }

    pub fn generate_shared_for_task(&self, task: &RticTask) -> TokenStream2 {
        let cfg_core = multibin::multibin_cfg_core(self.args.core);
        let task_resources_idents = &task.args.shared_idents;
        if task_resources_idents.is_empty() {
            return quote!();
        }

        // generate `field_name : proxy_type` to use for populating struct body
        let field_and_proxytype = task_resources_idents.iter().filter_map(|resource_ident| {
            if let Some(resource) = self.get_field(resource_ident) {
                let ident = &resource.ident;
                let proxy_type = utils::get_proxy_name(ident);
                Some(quote! {#ident: #proxy_type})
            } else {
                None
            }
        });
        let field_and_proxytype2 = field_and_proxytype.clone();

        // TODO: replace `shared(&self)` with individual `shared_resource_name(&self) -> proxy_type`

        let task_ty = task.name();
        let task_prio = task.args.priority;
        let task_shared_resources_struct =
            format_ident!("__{}_shared_resources", task.name_snakecase());
        quote! {
            // Shared resources access through shared() API for `#task_ty`
            #cfg_core
            impl #task_ty {
                pub fn shared(&self) -> #task_shared_resources_struct {
                    const TASK_PRIORITY: u16 = #task_prio;
                    #task_shared_resources_struct::new(TASK_PRIORITY)
                }
            }

            // internal struct for `#task_ty` resource proxies
            #cfg_core
            struct #task_shared_resources_struct {
                #(pub #field_and_proxytype ,)*
            }

            #cfg_core
            impl #task_shared_resources_struct {
                #[inline(always)]
                pub fn new(priority: u16) -> Self {
                    Self {
                        #(#field_and_proxytype2::new(priority) ,)*
                    }
                }
            }

        }
    }
}

pub mod utils {
    use quote::format_ident;

    #[inline(always)]
    pub fn get_proxy_name(ident: &syn::Ident) -> syn::Ident {
        format_ident!("__{ident}_mutex")
    }
}
