use std::{cell::RefCell, collections::HashMap, rc::Rc};

use indexmap::IndexMap;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Generics, Ident};

use crate::{
    ast::{
        class::{ClassDefinition, MacroBlock},
        items::ClassItem,
    },
    repr::{
        graph::DependencyGraph,
        items::{
            LocalClassFieldInformation, LocalClassMethodInformation, StaticClassFieldInformation,
            StaticClassMethodInformation,
        },
        keywords::{CLASS_CONTINER_KW, CONSTRUCTOR_KW},
    },
};

#[derive(Debug)]
pub struct MacroInformation {
    pub(super) classes: HashMap<String, RefCell<ClassInformation>>,
}

impl MacroInformation {
    pub fn pre_comp(block: MacroBlock) -> syn::Result<Self> {
        let mut output = MacroInformation::construct(block)?;
        let dep_graph = DependencyGraph::construct_dependency_graph(&output)?;
        let findable = dep_graph.waterfall_dependencies(&mut output)?;
        if findable.len() != output.classes.keys().len() {
            // There exists a cycle in the graph
            let mut error: Option<syn::Error> = None;
            for (key, class) in output.classes {
                if !findable.contains(&key) {
                    let key_error = syn::Error::new(
                        class.borrow().ident.span(),
                        format!("`{}` is part of an inheritance cycle.", key),
                    );
                    match error {
                        Some(ref mut e) => e.combine(key_error),
                        None => error = Some(key_error),
                    }
                }
            }
            if let Some(full_error) = error {
                return Err(full_error);
            } else {
                panic!(
                    "Internal State Error: Dependency graph has a cycle but no errors were generated."
                )
            }
        }
        Ok(output)
    }

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
                RefCell::new(ClassInformation::construct(class)?),
            );
        }
        Ok(MacroInformation { classes })
    }

    pub fn compile(&self) -> TokenStream2 {
        let MacroInformation { classes } = self;

        let class_container = format_ident!("{}", CLASS_CONTINER_KW);

        let compiled_classes: Vec<TokenStream2> =
            classes.values().map(|x| x.borrow().compile()).collect();
        let compiled_exports: Vec<TokenStream2> = classes
            .values()
            .map(|x| x.borrow().compile_export(&class_container))
            .collect();

        quote! {
            mod #class_container {
                #(#compiled_classes)*
            }
            #(#compiled_exports)*
        }
    }
}

#[derive(Debug)]
pub struct ClassInformation {
    pub(super) ident: Ident,
    is_public: bool,
    pub(super) parent: Option<String>,
    generics: Generics,
    static_fields: HashMap<String, Rc<StaticClassFieldInformation>>,
    /// Local fields must retain order because the default constructor has to be deterministic
    local_fields: IndexMap<String, Rc<LocalClassFieldInformation>>,
    static_methods: HashMap<String, Rc<StaticClassMethodInformation>>,
    local_methods: HashMap<String, Rc<LocalClassMethodInformation>>,
}

impl ClassInformation {
    pub fn construct(class: ClassDefinition) -> syn::Result<Self> {
        let ident = class.class_ident;
        let is_public = !class.pub_kw.is_none();
        let parent = class.parent.map(|x| x.to_string());
        let generics = class.generics.clone();
        let mut static_fields = HashMap::new();
        let mut local_fields = IndexMap::new();
        let mut static_methods = HashMap::new();
        let mut local_methods = HashMap::new();
        let mut encountered_constructor = false;
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
                        local_fields.insert(
                            string,
                            Rc::new(LocalClassFieldInformation::construct(field)?),
                        );
                    } else {
                        if static_fields.contains_key(&string) {
                            return Err(syn::Error::new(
                                field.ident.span(),
                                format!("Static field `{}` is already defined", string),
                            ));
                        }
                        static_fields.insert(
                            string,
                            Rc::new(StaticClassFieldInformation::construct(field)?),
                        );
                    }
                }
                ClassItem::ClassConstructor(constructor) => {
                    if encountered_constructor {
                        return Err(syn::Error::new(
                            constructor.ident.span(),
                            "Cannot declare multiple constructors.",
                        ));
                    }
                    let owned_kw = CONSTRUCTOR_KW.to_owned();
                    if static_methods.contains_key(&owned_kw) {
                        // Might be better to do the ident check ahead of this but it's ok for now
                        return Err(syn::Error::new(
                            constructor.ident.span(),
                            format!(
                                "Static function named `{}` (which this will be mapped to) already exists.",
                                CONSTRUCTOR_KW
                            ),
                        ));
                    }
                    static_methods.insert(
                        owned_kw,
                        Rc::new(StaticClassMethodInformation::construct_from_constructor(
                            constructor,
                            &ident.to_string(),
                        )?),
                    );
                    encountered_constructor = true;
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
                        local_methods.insert(
                            string,
                            Rc::new(LocalClassMethodInformation::construct(method)?),
                        );
                    } else {
                        if static_methods.contains_key(&string) {
                            return Err(syn::Error::new(
                                method.ident.span(),
                                format!("Static method `{}` is already defined", string),
                            ));
                        }
                        static_methods.insert(
                            string,
                            Rc::new(StaticClassMethodInformation::construct_from_method(method)?),
                        );
                    }
                }
            }
        }

        Ok(ClassInformation {
            ident,
            is_public,
            parent,
            generics,
            static_fields,
            local_fields,
            static_methods,
            local_methods,
        })
    }

    pub fn add_parent_information(
        &mut self,
        parent_ref: &RefCell<ClassInformation>,
    ) -> syn::Result<()> {
        // Process local fields, these cannot be overwritten
        let parent = parent_ref.borrow();
        Self::add_inhereted_loc_fields(&parent.local_fields, &mut self.local_fields)?;
        Self::add_inhereted_overridable(&parent.static_fields, &mut self.static_fields)?;
        Self::add_inhereted_overridable(&parent.local_methods, &mut self.local_methods)?;
        Self::add_inhereted_overridable(&parent.static_methods, &mut self.static_methods)?;
        Ok(())
    }

    fn add_inhereted_overridable<T>(
        parent_items: &HashMap<String, Rc<T>>,
        self_item: &mut HashMap<String, Rc<T>>,
    ) -> syn::Result<()>
    where
        T: InheritableItem,
    {
        for (name, field) in parent_items {
            if let Some(item) = self_item.get(name) {
                // TODO: Overriding
                return Err(syn::Error::new(
                    item.get_ident().span(),
                    format!("Overriding `{}` is not supported.", name),
                ));
            }
            self_item.insert(name.clone(), field.clone());
        }
        Ok(())
    }

    fn add_inhereted_loc_fields<T>(
        parent_items: &IndexMap<String, Rc<T>>,
        self_item: &mut IndexMap<String, Rc<T>>,
    ) -> syn::Result<()>
    where
        T: InheritableItem,
    {
        for (name, field) in parent_items {
            if let Some(item) = self_item.get(name) {
                return Err(syn::Error::new(
                    item.get_ident().span(),
                    format!("Overriding `{}` is not allowed.", name),
                ));
            }
            self_item.insert(name.clone(), field.clone());
        }
        Ok(())
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
            .map(|x| x.compile(generics, ident))
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
            pub mod #ident {
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

    pub fn compile_export(&self, class_container_ident: &Ident) -> TokenStream2 {
        let ClassInformation { ident, .. } = self;
        quote! { use #class_container_ident :: #ident ; }
    }
}

pub trait InheritableItem {
    fn get_ident(&self) -> &Ident;
    // TODO: Add type equivalence checking.
}
