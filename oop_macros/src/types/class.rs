use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    braced, custom_keyword,
    parse::{Parse, ParseStream},
};

use crate::types::{
    fields::ClassField, modifiers::ClassVisibility, utils::parse_zero_or_more, validate::Validate,
};

custom_keyword!(class);

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
        let fields = parse_zero_or_more::<ClassField>(&content)?;

        let output = ClassDef {
            attrs,
            vis,
            ident,
            generics,
            fields,
        };
        output.validate()?;
        Ok(output)
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

        let protected_ident = format_ident!("__{}Protected", ident);
        tokens.extend(quote! {
            #(#attrs)*
            #vis struct #ident #generics {
                #(#in_struct)*
            }

            impl #ident #generics {
                #(#in_trait)*
            }

            trait #protected_ident #generics {
                #(#in_protected)*
            }

            impl #protected_ident #generics for #ident {}
        });
    }
}

impl Validate for ClassDef {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized,
    {
        for field in &self.fields {
            field.validate()?;
        }
        Ok(())
    }
}
