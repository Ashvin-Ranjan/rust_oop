use std::collections::HashSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
};

use crate::types::{
    modifiers::{ClassModifiers, ClassVisibility},
    validate::Validate,
};

#[derive(Debug)]
pub enum ClassField {
    ClassAttribute(ClassAttribute),
    ClassMethod(ClassMethod),
}

impl Parse for ClassField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        return Ok(ClassField::ClassMethod(input.parse::<ClassMethod>()?));
        // Step 1: fork the input to try ClassMethod
        // let forked = input.fork();
        // if let Ok(method) = forked.parse::<ClassMethod>() {
        //     // Parsing succeeded on the fork, commit it
        //     input.advance_to(&forked);
        //     return Ok(ClassField::ClassMethod(method));
        // }

        // // Step 2: fallback to ClassAttribute
        // let attr = input.parse::<ClassAttribute>()?;
        // Ok(ClassField::ClassAttribute(attr))
    }
}

impl ToTokens for ClassField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            ClassField::ClassMethod(method) => quote! { #method },
            ClassField::ClassAttribute(attr) => quote! {},
        });
    }
}

impl Validate for ClassField {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        match self {
            ClassField::ClassAttribute(attribute) => attribute.validate(),
            ClassField::ClassMethod(method) => method.validate(),
        }
    }
}

#[derive(Debug)]
pub struct ClassAttribute {
    pub modifiers: ClassModifiers,
    pub attrs: Vec<syn::Attribute>,
    pub vis: ClassVisibility,
    pub mutability: syn::FieldMutability,
    pub is_static: bool,
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

impl Parse for ClassAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!("ClassAttribute not implemented yet.")
    }
}

impl Validate for ClassAttribute {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        todo!("ClassAttribute not implemented yet.")
    }
}

#[derive(Debug)]
pub struct ClassMethod {
    pub modifiers: ClassModifiers,
    pub attrs: Vec<syn::Attribute>,
    pub vis: ClassVisibility,
    pub sig: syn::Signature,
    pub block: Box<syn::Block>,
}

impl Parse for ClassMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<ClassVisibility>()?;
        let modifiers = input.parse::<ClassModifiers>()?;
        let mut sig = input.call(syn::Signature::parse)?;
        let block = Box::new(input.parse::<syn::Block>()?);
        if !modifiers.is_static && !sig.receiver().is_some() {
            sig.inputs.insert(0, parse_quote!(&mut self));
        }

        Ok(ClassMethod {
            modifiers,
            attrs,
            vis,
            sig,
            block,
        })
    }
}

impl ToTokens for ClassMethod {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ClassMethod {
            modifiers: _,
            attrs,
            vis,
            sig,
            block,
        } = self;

        tokens.extend(quote! {
            #(#attrs)*
            #vis #sig #block
        });
    }
}

impl Validate for ClassMethod {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        self.modifiers.validate()?;
        if let Some(reciever) = self.sig.receiver() {
            if self.modifiers.is_static {
                return Err(syn::Error::new(
                    reciever.span(),
                    "Static class methods cannot have a reciever.",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ClassConstructor {
    pub constructor: ClassMethod,
}
