use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Token, braced, custom_keyword,
    parse::{Parse, ParseStream, discouraged::Speculative},
    parse_quote,
};
custom_keyword!(prot);
custom_keyword!(class);

fn parse_zero_or_more<T: Parse>(input: ParseStream) -> Vec<T> {
    let mut result = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    return result;
}

#[derive(Debug)]
pub struct ClassDef {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub fields: Vec<ClassField>,
}

impl Parse for ClassDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.call(syn::Visibility::parse)?;
        input.parse::<class>()?;
        let ident = input.parse::<syn::Ident>()?;
        let generics = input.call(syn::Generics::parse)?;

        let content;
        braced!(content in input);
        let fields = parse_zero_or_more::<ClassField>(&content);
        Ok(ClassDef {
            attrs,
            vis,
            ident,
            generics,
            fields,
        })
    }
}

impl ToTokens for ClassDef {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ClassDef {
            attrs,
            vis,
            ident,
            generics,
            fields,
        } = self;
        let mut in_struct: Vec<&ClassField> = Vec::new();
        let mut in_trait: Vec<&ClassField> = Vec::new();
        let mut in_protected: Vec<&ClassField> = Vec::new();
        for field in fields {
            match field {
                ClassField::ClassAttribute(attr) => {
                    // if attr.vis == ClassVisibility::Protected {
                    //     in_protected.push(attr);
                    // } else if attr.modifiers.contains(&ClassModifiers::Static)
                    //     || attr.modifiers.contains(&ClassModifiers::Constant)
                    // {
                    //     in_trait.push(attr);
                    // } else {
                    //     in_struct.push(attr);
                    // }
                }
                ClassField::ClassMethod(method) => {
                    if method.vis == ClassVisibility::Protected {
                        in_protected.push(field);
                    } else {
                        in_trait.push(field);
                    }
                }
            }
        }

        tokens.extend(quote! {
           #(#attrs)*
           #vis struct #ident #generics {
               #(#in_struct)*
           }

           impl #ident #generics {
               #(#in_trait)*
           }
        });
    }
}

#[derive(PartialEq, Debug)]
enum ClassVisibility {
    Public,
    Protected,
    Private,
}

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

#[derive(PartialEq, Debug)]
enum ClassModifiers {
    Constant,
    Static,
}

impl Parse for ClassModifiers {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![const]) {
            input.parse::<Token![const]>()?;
            Ok(Self::Constant)
        } else if input.peek(Token![static]) {
            input.parse::<Token![static]>()?;
            Ok(Self::Static)
        } else {
            Err(input.error("expected `const` or `static`"))
        }
    }
}

#[derive(Debug)]
enum ClassField {
    ClassAttribute(ClassAttribute),
    ClassMethod(ClassMethod),
}

impl Parse for ClassField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        return Ok(ClassField::ClassMethod(input.parse::<ClassMethod>()?));
        // Step 1: fork the input to try ClassMethod
        let forked = input.fork();
        if let Ok(method) = forked.parse::<ClassMethod>() {
            // Parsing succeeded on the fork, commit it
            input.advance_to(&forked);
            return Ok(ClassField::ClassMethod(method));
        }

        // Step 2: fallback to ClassAttribute
        let attr = input.parse::<ClassAttribute>()?;
        Ok(ClassField::ClassAttribute(attr))
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

#[derive(Debug)]
struct ClassAttribute {
    pub modifiers: Vec<ClassModifiers>,
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

#[derive(Debug)]
struct ClassMethod {
    pub modifiers: Vec<ClassModifiers>,
    pub attrs: Vec<syn::Attribute>,
    pub vis: ClassVisibility,
    pub sig: syn::Signature,
    pub block: Box<syn::Block>,
}

impl Parse for ClassMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<ClassVisibility>()?;
        let modifiers = parse_zero_or_more::<ClassModifiers>(input);
        let mut sig = input.call(syn::Signature::parse)?;
        let block = Box::new(input.parse::<syn::Block>()?);
        if !modifiers.contains(&ClassModifiers::Static) && !sig.receiver().is_some() {
            sig.inputs.insert(0, parse_quote!(&mut self));
        } else if modifiers.contains(&ClassModifiers::Static) && sig.receiver().is_some() {
            return Err(syn::Error::new_spanned(
                sig.receiver(),
                "Static methods are not allowed to contain a reciever",
            ));
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

struct ClassConstructor {
    pub constructor: ClassMethod,
}
