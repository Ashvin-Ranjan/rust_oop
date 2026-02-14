use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Generics, Ident, spanned::Spanned};

use crate::{
    ast::{
        class::{ClassDefinition, MacroBlock},
        items::ClassItem,
    },
    repr::items::{
        LocalClassFieldInformation, LocalClassMethodInformation, StaticClassFieldInformation,
        StaticClassMethodInformation,
    },
};

#[derive(Debug)]
pub struct MacroInformation {
    classes: HashMap<String, ClassInformation>,
}

impl MacroInformation {
    pub fn construct(block: MacroBlock) -> syn::Result<Self> {
        let mut classes = HashMap::new();
        for class in block.definitions {
            if classes.contains_key(&class.class_ident.to_string()) {
                return Err(syn::Error::new(
                    class.class_ident.span(),
                    format!("Class `{}` has already been defined.", class.class_ident),
                ));
            }
            classes.insert(
                class.class_ident.to_string(),
                ClassInformation::construct(class)?,
            );
        }
        Ok(MacroInformation { classes })
    }

    pub fn compile(&self) -> TokenStream2 {
        let MacroInformation { classes } = self;

        let compiled_classes: Vec<TokenStream2> = classes.values().map(|x| x.compile()).collect();

        quote! {
            #(#compiled_classes)*
        }
    }
}

#[derive(Debug)]
pub struct ClassInformation {
    ident: Ident,
    is_public: bool,
    parents: HashSet<String>,
    generics: Generics,
    static_fields: HashMap<String, StaticClassFieldInformation>,
    /// Local fields must retain order because the default constructor has to be deterministic
    local_fields: IndexMap<String, LocalClassFieldInformation>,
    static_methods: HashMap<String, StaticClassMethodInformation>,
    local_methods: HashMap<String, LocalClassMethodInformation>,
}

impl ClassInformation {
    pub fn construct(class: ClassDefinition) -> syn::Result<Self> {
        let ident = class.class_ident;
        let is_public = !class.pub_kw.is_none();
        let mut parents = HashSet::new();

        if !class.parents.is_empty() {
            return Err(syn::Error::new(
                class.parents.span(),
                "Inheritance is currently not supported.",
            ));
        }

        for parent in &class.parents {
            if parents.contains(&parent.to_string()) {
                return Err(syn::Error::new(
                    parent.span(),
                    format!("Cannot inherit from `{}` twice.", parent),
                ));
            }
            parents.insert(parent.to_string());
        }
        let generics = class.generics.clone();
        let mut static_fields = HashMap::new();
        let mut local_fields = IndexMap::new();
        let mut static_methods = HashMap::new();
        let mut local_methods = HashMap::new();
        for item in class.items {
            match item {
                ClassItem::ClassField(field) => {
                    let string = field.ident.to_string();
                    if field.static_kw.is_none() {
                        if local_fields.contains_key(&string) {
                            return Err(syn::Error::new(
                                field.ident.span(),
                                format!("Local field `{}` is already defined", string),
                            ));
                        }
                        local_fields.insert(string, LocalClassFieldInformation::construct(field)?);
                    } else {
                        if static_fields.contains_key(&string) {
                            return Err(syn::Error::new(
                                field.ident.span(),
                                format!("Static field `{}` is already defined", string),
                            ));
                        }
                        static_fields
                            .insert(string, StaticClassFieldInformation::construct(field)?);
                    }
                }
                ClassItem::ClassConstructor(constructor) => {
                    return Err(syn::Error::new(
                        constructor.ident.span(),
                        "Constructors are currently not supported, use static functions instead.",
                    ));
                }
                ClassItem::ClassMethod(method) => {
                    let string = method.ident.to_string();
                    if method.static_kw.is_none() {
                        if local_methods.contains_key(&string) {
                            return Err(syn::Error::new(
                                method.ident.span(),
                                format!("Local method `{}` is already defined", string),
                            ));
                        }
                        local_methods
                            .insert(string, LocalClassMethodInformation::construct(method)?);
                    } else {
                        if static_methods.contains_key(&string) {
                            return Err(syn::Error::new(
                                method.ident.span(),
                                format!("Static method `{}` is already defined", string),
                            ));
                        }
                        static_methods
                            .insert(string, StaticClassMethodInformation::construct(method)?);
                    }
                }
                _ => {}
            }
        }

        Ok(ClassInformation {
            ident,
            is_public,
            parents,
            generics,
            static_fields,
            local_fields,
            static_methods,
            local_methods,
        })
    }

    pub fn compile(&self) -> TokenStream2 {
        let ClassInformation {
            ident,
            is_public,
            generics,
            static_fields,
            local_fields,
            static_methods,
            local_methods,
            ..
        } = self;

        let class_visibility = if *is_public {
            quote! {pub}
        } else {
            quote! {}
        };

        let where_clause = &generics.where_clause;

        let compiled_static_fields: Vec<TokenStream2> =
            static_fields.values().map(|x| x.compile()).collect();

        let compiled_local_fields: Vec<TokenStream2> =
            local_fields.values().map(|x| x.compile(true)).collect();

        let compiled_local_fields_args: Vec<TokenStream2> =
            local_fields.values().map(|x| x.compile(false)).collect();

        let compiled_local_fields_names: Vec<TokenStream2> = local_fields
            .values()
            .map(|x| x.compile_as_names())
            .collect();

        let compiled_static_methods: Vec<TokenStream2> = static_methods
            .values()
            .map(|x| x.compile(generics))
            .collect();

        let compiled_local_methods: Vec<TokenStream2> =
            local_methods.values().map(|x| x.compile()).collect();

        let mut i = 0u32;
        while local_fields.contains_key(&format!("__phantom_marker{}", i)) {
            i += 1;
        }
        let phantom_marker = format_ident!("__phantom_marker{}", i);
        let params = &generics.params;

        quote! {
            #[allow(non_snake_case)]
            mod #ident {
                #class_visibility struct #ident #generics #where_clause {
                    #(#compiled_local_fields ,)*
                    #phantom_marker: ::std::marker::PhantomData<(#params)>
                }
                impl #generics #ident #generics #where_clause {
                    #(#compiled_local_methods)*
                }
                #(#compiled_static_fields)*
                #(#compiled_static_methods)*
                fn _default_constructor #generics ( #(#compiled_local_fields_args ,)* ) -> #ident #generics #where_clause {
                    #ident {
                        #(#compiled_local_fields_names ,)*
                        #phantom_marker : ::std::marker::PhantomData,
                    }
                }
            }
        }
    }
}
