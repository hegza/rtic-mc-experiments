mod codegen;
// mod error;
mod parse;

use std::cell::RefCell;

use codegen::Codegen;
use parse::App;
use proc_macro2::TokenStream;
use rtic_core::RticPass;
use rtic_core::parse_utils::RticAttr;
use syn::ItemMod;

pub const PCS_ATTR_IDENT: &str = "fast";

// HACK: pass the list of dispatchers from PCS pass to core pass backend.
thread_local!(pub static PCS_DISPATCHERS: RefCell<Vec<proc_macro2::Ident>> = const { RefCell::new(vec![]) });

pub struct PcsPass {
    max_num_pcs: usize,
}

impl PcsPass {
    /// Use `max_num_pcs` to specify the number of hardware PCS slots provided by the platform
    pub fn new(max_num_pcs: usize) -> Self {
        Self { max_num_pcs }
    }
}

impl RticPass for PcsPass {
    fn run_pass(&self, args: TokenStream, app_mod: ItemMod) -> syn::Result<(TokenStream, ItemMod)> {
        let params = RticAttr::parse_from_tokens(args.clone())?;
        let mut parsed = App::parse(&params, app_mod)?;

        self.analyze(&mut parsed);

        let code = Codegen::new(parsed).run();
        Ok((args, code))
    }

    fn pass_name(&self) -> &str {
        "pcs-pass"
    }
}

impl PcsPass {
    fn analyze(&self, app: &mut App) {
        // Partition interrupts into PCS interrupts and
        // non-PCS interrupts based on the presence of the attribute.
        let (fast_tasks, other_tasks): (Vec<_>, Vec<_>) =
            app.tasks.iter().partition(|task| task.fast);

        // Limit to maximum number of PCS interrupts supported by hardware
        if fast_tasks.len() > self.max_num_pcs {
            panic!(
                "Exceeded number of interrupts leveraging PCS for this platform ({}), please reduce the number of accelerated tasks\nFast IRQs: {:?}\nOther IRQs: {:?}",
                self.max_num_pcs,
                fast_tasks
                    .iter()
                    .map(|task| format!("{} ({})", task.name, task.binds))
                    .collect::<Vec<_>>(),
                other_tasks
                    .iter()
                    .map(|task| task.name.clone())
                    .collect::<Vec<_>>()
            );
        }

        let pcs_dispatchers = fast_tasks
            // Save bound interrupt for later processing
            .map(|task| task.binds.clone())
            .collect();
        PCS_DISPATCHERS.replace(pcs_dispatchers);
    }
}
