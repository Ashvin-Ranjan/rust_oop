use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Ident, ReturnType, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::types::{modifiers::ClassModifiers, validate::Validate};

#[derive(Debug)]
pub struct MethodSignature {
    identifier: Ident,
    function_args: Punctuated<IdentType, Token![,]>,
    return_type: ReturnType,
    // This are needed during reconstruction
    is_static: bool,
    is_constant: bool,
}

impl Parse for MethodSignature {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![fn]>()?;
        let identifier = input.parse::<Ident>()?;
        let content;
        parenthesized!(content in input);
        let function_args = content.call(Punctuated::<IdentType, Token![,]>::parse_terminated)?;
        let return_type = input.parse::<ReturnType>()?;

        Ok(MethodSignature {
            identifier,
            function_args,
            return_type,
            is_static: false,
            is_constant: false,
        })
    }
}

impl ToTokens for MethodSignature {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let MethodSignature {
            identifier,
            function_args,
            return_type,
            is_static,
            is_constant,
        } = self;

        tokens.extend(quote! { fn #identifier });
        if !is_static {
            if *is_constant {
                tokens.extend(quote! { (&self, #function_args) })
            } else {
                tokens.extend(quote! { (&mut self, #function_args) })
            }
        } else {
            tokens.extend(quote! { (#function_args) })
        }
        tokens.extend(quote! { #return_type});
    }
}

impl Validate for MethodSignature {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl MethodSignature {
    pub fn update_information(&mut self, modifiers: &ClassModifiers) {
        self.is_constant = modifiers.is_constant;
        self.is_static = modifiers.is_static;
    }
}

#[derive(Debug)]
pub struct IdentType {
    pub ident: Ident,
    pub ty: Box<Type>,
}

impl Parse for IdentType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(IdentType { ident, ty })
    }
}

impl ToTokens for IdentType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let IdentType { ident, ty } = self;
        tokens.extend(quote! { #ident : #ty });
    }
}

#[derive(Debug)]
pub struct ConstructorSignature {
    pub identifier: Ident,
    function_args: Punctuated<IdentType, Token![,]>,
}

impl Parse for ConstructorSignature {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let identifier = input.parse::<Ident>()?;
        let content;
        parenthesized!(content in input);
        let function_args = content.call(Punctuated::<IdentType, Token![,]>::parse_terminated)?;

        Ok(ConstructorSignature {
            identifier,
            function_args,
        })
    }
}

impl ToTokens for ConstructorSignature {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ConstructorSignature { function_args, .. } = self;

        tokens.extend(quote! { fn init (#function_args) -> Self });
    }
}

impl Validate for ConstructorSignature {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}
