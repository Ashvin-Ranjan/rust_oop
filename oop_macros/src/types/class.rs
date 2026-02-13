use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    braced, custom_keyword,
    parse::{Parse, ParseStream},
};

use crate::types::{
    fields::{ClassAttribute, ClassField},
    modifiers::ClassVisibility,
    utils::parse_zero_or_more,
    validate::Validate,
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
        let mut generics = input.call(syn::Generics::parse)?;
        generics.where_clause = input.parse()?;

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
        let mut in_struct: Vec<&ClassAttribute> = Vec::new();
        let mut in_trait: Vec<&ClassField> = Vec::new();
        let mut in_protected: Vec<&ClassField> = Vec::new();
        for field in fields {
            match field {
                ClassField::ClassAttribute(attr) => {
                    if attr.vis == ClassVisibility::Protected {
                        in_protected.push(field);
                    } else if attr.modifiers.is_constant || attr.modifiers.is_static {
                        in_trait.push(field);
                    } else {
                        in_struct.push(attr);
                    }
                }
                ClassField::ClassMethod(method) => {
                    if method.vis == ClassVisibility::Protected {
                        in_protected.push(field);
                    } else {
                        in_trait.push(field);
                    }
                }
                ClassField::ClassConstructor(_) => {
                    in_trait.push(field);
                }
            }
        }

        let protected_ident = format_ident!("__{}Protected", ident);
        let generic_params = &generics.params;
        let where_clause = &generics.where_clause;
        let signatures = convert_fields_to_signature(&in_struct);
        let idents = convert_fields_to_idents(&in_struct);

        tokens.extend(quote! {
            #(#attrs)*
            #vis struct #ident #generics #where_clause {
                #(#in_struct ,)*
                __phantom_markers: ::std::marker::PhantomData<(#generic_params)>
            }

            impl #generics #ident #generics #where_clause {
                #(#in_trait)*
                fn _default_constructor(#(#signatures ,)*) -> Self {
                    #ident {
                        #(#idents ,)*
                        __phantom_markers: ::std::marker::PhantomData
                    }
                }
            }

            trait #protected_ident #generics #where_clause {
                #(#in_protected)*
            }

            impl #generics #protected_ident #generics for #ident #generics #where_clause {}
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
            if let ClassField::ClassConstructor(cons) = field {
                if cons.sig.identifier != self.ident {
                    return Err(syn::Error::new(
                        cons.sig.identifier.span(),
                        "Constructor must share name of class.",
                    ));
                }
            }
        }
        Ok(())
    }
}

struct SignatureField {
    ident: syn::Ident,
    ty: syn::Type,
}

impl ToTokens for SignatureField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let SignatureField { ident, ty } = self;
        tokens.extend(quote! { #ident : #ty });
    }
}

fn convert_fields_to_signature(fields: &Vec<&ClassAttribute>) -> Vec<SignatureField> {
    fields
        .iter()
        .map(|x| SignatureField {
            ident: x.ident.clone(),
            ty: x.ty.clone(),
        })
        .collect()
}

fn convert_fields_to_idents(fields: &Vec<&ClassAttribute>) -> Vec<syn::Ident> {
    fields.iter().map(|x| x.ident.clone()).collect()
}
