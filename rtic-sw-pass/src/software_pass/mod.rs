mod analyze;
mod codegen;
pub(crate) mod parse;

use crate::parse::ParsedApp;
use analyze::AppAnalysis;
use proc_macro2::TokenStream;
use rtic_core::parse_utils::RticAttr;
use rtic_core::RticPass;

pub struct ScSoftwarePass {}

impl RticPass for ScSoftwarePass {
    fn run_pass(&self, params: RticAttr, app_mod: TokenStream) -> syn::Result<TokenStream> {
        let parsed = ParsedApp::parse(&params, syn::parse2(app_mod)?)?;
        let analysis = AppAnalysis::run(&parsed)?;
        let code = codegen::generate(&parsed, &analysis);
        // [ ] we still need some traits for hw specific impl for pend() and cross_pend() !
        Ok(code)
    }
}
