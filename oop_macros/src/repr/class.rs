use std::{cell::RefCell, collections::HashMap, rc::Rc};

use indexmap::IndexMap;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{AngleBracketedGenericArguments, GenericArgument, Generics, Ident, spanned::Spanned};

use crate::{
    ast::{
        class::{ClassDefinition, MacroBlock},
        items::ClassItem,
    },
    repr::{
        generics::{generic_param_to_string, map_generic_arg},
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
        for class in classes.keys() {
            if let Some(invalid_class) = classes.get(&format!("{}Instance", class)) {
                return Err(syn::Error::new(
                    invalid_class.borrow().ident.span(),
                    format!(
                        "`{}Instance` is a reserved name (see technical details for more information on why).",
                        class
                    ),
                ));
            }
        }
        Ok(MacroInformation { classes })
    }

    pub fn compile(&self) -> TokenStream2 {
        let MacroInformation { classes } = self;

        let class_container = format_ident!("{}", CLASS_CONTINER_KW);

        let compiled_classes: Vec<TokenStream2> =
            classes.values().map(|x| x.borrow().compile(self)).collect();

        let imports: Vec<TokenStream2> = classes
            .keys()
            .map(|name| {
                let ident = format_ident!("{}", name);
                quote! { use #class_container :: #ident :: *; }
            })
            .collect();

        quote! {
            mod #class_container {
                #(#compiled_classes)*
            }
            #(#imports)*
        }
    }

    pub fn compile_imports(&self, exclude: String) -> Vec<TokenStream2> {
        let MacroInformation { classes } = self;
        classes
            .keys()
            .filter(|name| **name != exclude)
            .map(|name| {
                let ident = format_ident!("{}", name);
                quote! { use super:: #ident :: *; }
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct ClassInformation {
    pub(super) ident: Ident,
    is_public: bool,
    pub(super) parent: Option<String>,
    generics: Generics,
    parent_generics: Option<AngleBracketedGenericArguments>,
    parent_generic_mapping: Rc<RefCell<HashMap<String, GenericArgument>>>,
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
        let generics = class.generics;
        let parent_generics = class.parent_generics;
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
            parent_generics,
            parent_generic_mapping: Rc::new(RefCell::new(HashMap::new())),
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
        let parent = parent_ref.borrow();

        // Handle generics
        if let Some(parent_args) = &self.parent_generics {
            if parent_args.args.len() != parent.generics.params.len() {
                return Err(syn::Error::new(
                    parent_args.args.span(),
                    format!(
                        "Parent requires {} parameters but {} were provided.",
                        parent.generics.params.len(),
                        parent_args.args.len()
                    ),
                ));
            }
            for (param, arg) in parent.generics.params.iter().zip(parent_args.args.iter()) {
                let param_name = generic_param_to_string(param);
                self.parent_generic_mapping
                    .borrow_mut()
                    .insert(param_name, arg.clone());
            }
        } else if !parent.generics.params.is_empty() {
            return Err(syn::Error::new(
                self.ident.span(),
                format!(
                    "Parent `{}` requires {} generic argument(s) but none were provided.",
                    parent.ident,
                    parent.generics.params.len()
                ),
            ));
        }

        // Add in fields
        Self::add_inhereted_loc_fields(&parent.local_fields, &mut self.local_fields)?;
        Self::add_inhereted_overridable(&parent.static_fields, &mut self.static_fields)?;
        Self::add_inhereted_overridable(&parent.local_methods, &mut self.local_methods)?;
        Self::add_inhereted_overridable(&parent.static_methods, &mut self.static_methods)?;
        Ok(())
    }

    fn add_inhereted_overridable<T>(
        parent_items: &HashMap<String, Rc<T>>,
        self_items: &mut HashMap<String, Rc<T>>,
    ) -> syn::Result<()>
    where
        T: InheritableItem + OverridableItem,
    {
        for (name, field) in parent_items {
            if let Some(item) = self_items.get(name) {
                if !item.get_is_overriding() {
                    return Err(syn::Error::new(
                        item.get_ident().span(),
                        "Overriding requires use of the `override` keyword.",
                    ));
                }
                if !item.is_compatable_override(field) {
                    return Err(syn::Error::new(
                        item.get_ident().span(),
                        format!("Cannot override with an incompatible type."),
                    ));
                }
            } else {
                self_items.insert(name.clone(), field.clone());
            }
        }
        for (name, field) in self_items {
            if field.get_is_overriding() && !parent_items.contains_key(name) {
                return Err(syn::Error::new(
                    field.get_ident().span(),
                    "Override keyword is used but nothing is being overidden.",
                ));
            }
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
                    "Overriding local fields is not allowed.",
                ));
            }
            self_item.insert(name.clone(), field.clone());
        }
        Ok(())
    }

    pub fn compile(&self, macro_info: &MacroInformation) -> TokenStream2 {
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

        let trait_value = self.compile_trait();
        let inherited_traits = self.compile_inhereted_traits(macro_info);

        let imports = macro_info.compile_imports(ident.to_string());

        quote! {
            #[allow(non_snake_case)]
            pub mod #ident {
                #(#imports)*
                #class_visibility struct #ident #generics #where_clause {
                    #(#compiled_local_fields ,)*
                    #[doc(hidden)]
                    #phantom_marker: ::std::marker::PhantomData<(#params)>
                }
                impl #generics #ident #generics #where_clause {
                    #(#compiled_local_methods)*
                    #(#compiled_static_fields)*
                    #(#compiled_static_methods)*
                    fn _default_constructor ( #(#compiled_local_fields_args ,)* ) -> #ident #generics #where_clause {
                        #ident {
                            #(#compiled_local_fields_names ,)*
                            #phantom_marker : ::std::marker::PhantomData,
                        }
                    }
                }
                #trait_value
                #(#inherited_traits)*
            }
        }
    }

    fn compile_trait(&self) -> TokenStream2 {
        let ClassInformation {
            ident,
            parent,
            local_methods,
            generics,
            parent_generics,
            ..
        } = self;

        let trait_ident = format_ident!("{}Instance", ident);

        let trait_definitions: Vec<TokenStream2> = local_methods
            .values()
            .filter(|x| x.is_public())
            .map(|x| x.compile_for_trait_def())
            .collect();

        let trait_declarations: Vec<TokenStream2> = local_methods
            .values()
            .filter(|x| x.is_public())
            .map(|x| x.compile_for_trait_impl())
            .collect();

        let mut inherit = quote! {};
        if let Some(p) = parent {
            let parent_trait_ident = format_ident!("{}Instance", p);
            if let Some(pg) = parent_generics {
                inherit = quote! { : #parent_trait_ident #pg };
            } else {
                inherit = quote! { : #parent_trait_ident };
            }
        }

        let where_clause = &generics.where_clause;

        quote! {
            pub trait #trait_ident #generics #inherit #where_clause {
                #(#trait_definitions)*
            }
            impl #generics #trait_ident #generics for #ident #generics #where_clause {
                #(#trait_declarations)*
            }
        }
    }

    fn compile_inhereted_traits(&self, macro_info: &MacroInformation) -> Vec<TokenStream2> {
        let ClassInformation {
            ident,
            parent,
            generics,
            parent_generic_mapping,
            ..
        } = self;

        let mut compiled_traits = Vec::new();
        let mut curr_parent_op = parent.clone();

        // current_mapping: maps each direct parent's param name → GenericArgument from self's perspective
        let mut current_mapping: HashMap<String, GenericArgument> =
            parent_generic_mapping.borrow().clone();

        while let Some(curr_parent_name) = curr_parent_op {
            let parent_info = macro_info
                .classes
                .get(&curr_parent_name)
                .expect(
                    "Internal State Error: Unable to find class during parental implementations.",
                )
                .borrow();

            let parent_trait_ident = format_ident!("{}Instance", curr_parent_name);
            let trait_declarations: Vec<TokenStream2> = parent_info
                .local_methods
                .values()
                .filter(|x| x.is_public())
                .map(|x| x.compile_for_trait_impl())
                .collect();

            // Resolve each of this ancestor's params to a concrete arg from self's perspective
            let resolved_args: Vec<GenericArgument> = parent_info
                .generics
                .params
                .iter()
                .map(|p| {
                    let name = generic_param_to_string(p);
                    current_mapping.get(&name).cloned().unwrap_or_else(|| {
                        panic!(
                            "Internal State Error: Generic mapping missing param `{}`",
                            name
                        )
                    })
                })
                .collect();

            compiled_traits.push(quote! {
                impl #generics #parent_trait_ident <#(#resolved_args ,)*> for #ident #generics {
                    #(#trait_declarations)*
                }
            });

            // Compose: build next_mapping for grandparent by substituting current_mapping
            // into each value of parent's mapping to its own parent
            let mut next_mapping: HashMap<String, GenericArgument> = HashMap::new();
            for (gp_name, parent_arg) in parent_info.parent_generic_mapping.borrow().iter() {
                next_mapping.insert(
                    gp_name.clone(),
                    map_generic_arg(parent_arg.clone(), &current_mapping),
                );
            }
            current_mapping = next_mapping;
            curr_parent_op = parent_info.parent.clone();
        }

        compiled_traits
    }
}

pub trait InheritableItem {
    fn get_ident(&self) -> &Ident;
}

pub trait OverridableItem {
    fn get_is_overriding(&self) -> bool;
    fn is_compatable_override(&self, other: &Self) -> bool;
}
