use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod types;

#[proc_macro]
pub fn class(item: TokenStream) -> TokenStream {
    let class_value = parse_macro_input!(item as types::ClassDef);
    println!("{:#?}", class_value);
    println!("gurt! {}", quote! { #class_value });
    quote! { #class_value }.into()
}
