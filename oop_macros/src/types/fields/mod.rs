use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Block, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use crate::types::{
    fields::signature::{ConstructorSignature, MethodSignature},
    modifiers::{ClassModifiers, ClassVisibility},
    validate::Validate,
};

mod signature;

#[derive(Debug)]
pub enum ClassField {
    ClassAttribute(ClassAttribute),
    ClassMethod(ClassMethod),
    ClassConstructor(ClassConstructor),
}

impl Parse for ClassField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![let]) {
            Ok(ClassField::ClassAttribute(input.parse()?))
        } else {
            // Need to distinguish between method and constructor
            // Use fork to peek ahead
            let fork = input.fork();

            // Skip visibility & modifiers
            let a = fork.parse::<ClassVisibility>()?;
            fork.parse::<ClassModifiers>()?;

            let possible_method = fork.parse::<Option<Token![fn]>>()?;

            if possible_method.is_none() {
                // We have hit a constructor
                Ok(ClassField::ClassConstructor(input.parse()?))
            } else {
                Ok(ClassField::ClassMethod(input.parse()?))
            }
        }
    }
}

impl ToTokens for ClassField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            ClassField::ClassMethod(method) => quote! { #method },
            ClassField::ClassAttribute(attr) => quote! { #attr },
            ClassField::ClassConstructor(constructor) => quote! { #constructor },
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
            ClassField::ClassConstructor(constructor) => constructor.validate(),
        }
    }
}

#[derive(Debug)]
pub struct ClassAttribute {
    pub modifiers: ClassModifiers,
    pub attrs: Vec<syn::Attribute>,
    pub vis: ClassVisibility,
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub expression: Option<syn::Expr>,
    pub semicolon: syn::token::Semi,
}

impl Parse for ClassAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        input.parse::<Token![let]>()?;
        let vis = input.parse::<ClassVisibility>()?;
        let modifiers = input.parse::<ClassModifiers>()?;
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<syn::Type>()?;
        let mut expression = None;
        if modifiers.is_constant || modifiers.is_static {
            input.parse::<Token![=]>()?;
            expression = Some(input.parse()?);
        }
        let semicolon = input.parse()?;

        Ok(ClassAttribute {
            modifiers,
            attrs,
            vis,
            ident,
            ty,
            expression,
            semicolon,
        })
    }
}

impl ToTokens for ClassAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ClassAttribute {
            modifiers,
            attrs,
            vis,
            ident,
            ty,
            expression,
            ..
        } = self;
        if modifiers.is_constant || modifiers.is_static {
            tokens.extend(quote! { #(#attrs)* #vis const #ident : #ty = #expression ; });
        } else {
            tokens.extend(quote! { #(#attrs)* #vis #ident : #ty });
        }
    }
}

impl Validate for ClassAttribute {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        if (self.modifiers.is_constant || self.modifiers.is_static) && self.expression.is_none() {
            return Err(syn::Error::new(
                self.semicolon.span(),
                "Constant or static class fields must be assigned a value.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClassMethod {
    pub modifiers: ClassModifiers,
    pub attrs: Vec<syn::Attribute>,
    pub vis: ClassVisibility,
    pub sig: MethodSignature,
    pub block: Box<syn::Block>,
}

impl Parse for ClassMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<ClassVisibility>()?;
        let modifiers = input.parse::<ClassModifiers>()?;
        let mut sig = input.call(MethodSignature::parse)?;
        let block = Box::new(input.parse::<syn::Block>()?);
        sig.update_information(&modifiers);

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
        self.sig.validate()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClassConstructor {
    attrs: Vec<syn::Attribute>,
    vis: ClassVisibility,
    pub sig: ConstructorSignature,
    block: Box<Block>,
}

impl Parse for ClassConstructor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<ClassVisibility>()?;
        let sig = input.call(ConstructorSignature::parse)?;
        let block = Box::new(input.parse::<syn::Block>()?);

        Ok(ClassConstructor {
            attrs,
            vis,
            sig,
            block,
        })
    }
}

impl ToTokens for ClassConstructor {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ClassConstructor {
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

impl Validate for ClassConstructor {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}
