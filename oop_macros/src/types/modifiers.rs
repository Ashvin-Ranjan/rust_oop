use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Token, custom_keyword,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use crate::types::validate::Validate;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ClassVisibility {
    Public,
    Protected,
    Private,
}

custom_keyword!(prot);

impl Parse for ClassVisibility {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            Ok(Self::Public)
        } else if input.peek(prot) {
            input.parse::<prot>()?;
            Ok(Self::Protected)
        } else {
            Ok(Self::Private)
        }
    }
}

impl ToTokens for ClassVisibility {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            ClassVisibility::Private => tokens.extend(quote! { priv }),
            ClassVisibility::Public => tokens.extend(quote! { pub }),
            _ => {}
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ClassModifiers {
    pub is_constant: bool,
    pub is_static: bool,
    dup_constant: Vec<Token![const]>,
    dup_static: Vec<Token![static]>,
}

impl Parse for ClassModifiers {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut output = ClassModifiers {
            is_constant: false,
            is_static: false,
            dup_constant: Vec::new(),
            dup_static: Vec::new(),
        };
        while input.peek(Token![const]) || input.peek(Token![static]) {
            if input.peek(Token![const]) {
                if output.is_constant {
                    output.dup_constant.push(input.parse()?);
                } else {
                    output.is_constant = true;
                    input.parse::<Token![const]>()?;
                }
            } else if input.peek(Token![static]) {
                if output.is_static {
                    output.dup_static.push(input.parse()?);
                } else {
                    output.is_static = true;
                    input.parse::<Token![static]>()?;
                }
            }
        }
        Ok(output)
    }
}

impl Validate for ClassModifiers {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        if let Some(extra) = self.dup_constant.get(0) {
            return Err(syn::Error::new(extra.span(), "Duplicate modifier `const`."));
        }
        if let Some(extra) = self.dup_static.get(0) {
            return Err(syn::Error::new(
                extra.span(),
                "Duplicate modifier `static`.",
            ));
        }
        Ok(())
    }
}
