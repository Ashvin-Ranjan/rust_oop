use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::repr::class::MacroInformation;

mod ast;
mod repr;
mod types;
mod validate;

#[proc_macro]
pub fn class(item: TokenStream) -> TokenStream {
    let other = item.clone();
    let parsed_macro = parse_macro_input!(other as ast::class::MacroBlock);
    let macro_data = MacroInformation::construct(parsed_macro);
    match macro_data {
        Ok(macro_data) => {
            println!("{}", macro_data.compile());
            macro_data.compile().into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}
