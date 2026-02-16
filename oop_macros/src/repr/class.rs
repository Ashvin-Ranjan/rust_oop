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
            classes.values().map(|x| x.borrow().compile(self)).collect();

        quote! {
            mod #class_container {
                #(#compiled_classes)*
            }
            use #class_container :: *;
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
            parent,
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

        let mut parent_generics = None;
        if let Some(par) = parent {
            if let Some(parent_value) = macro_info.classes.get(par) {
                parent_generics = Some(parent_value.borrow().generics.clone());
            } else {
                panic!(
                    "Internal State Error: Attempting to inherit from class which does not exist."
                )
            }
        }

        let trait_value = self.compile_trait(parent_generics);
        let inherited_traits = self.compile_inhereted_traits(macro_info);

        quote! {
            #class_visibility struct #ident #generics #where_clause {
                #(#compiled_local_fields ,)*
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

    fn compile_trait(&self, parent_generics: Option<Generics>) -> TokenStream2 {
        let ClassInformation {
            ident,
            parent,
            local_methods,
            generics,
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
            inherit = quote! {: #parent_trait_ident}
        }

        let mut combined_generics = generics.clone();
        if let Some(p_gen) = &parent_generics {
            combined_generics = merge_generics(&combined_generics, p_gen);
        }

        let where_clause = &combined_generics.where_clause;

        quote! {
            pub trait #trait_ident #combined_generics #inherit #parent_generics #where_clause {
                #(#trait_definitions)*
            }
            impl #combined_generics #trait_ident #combined_generics for #ident #parent_generics #where_clause {
                #(#trait_declarations)*
            }
        }
    }

    fn compile_inhereted_traits(&self, macro_info: &MacroInformation) -> Vec<TokenStream2> {
        let ClassInformation { ident, parent, .. } = self;

        let mut compiled_traits = Vec::new();
        let mut curr_parent_op = parent.clone();
        while let Some(curr_parent) = curr_parent_op {
            let parent = macro_info
                .classes
                .get(&curr_parent)
                .expect(
                    "Internal State Error: Unable to find class during parental implementations.",
                )
                .borrow();

            let parent_trait_ident = format_ident!("{}Instance", curr_parent);
            let trait_declarations: Vec<TokenStream2> = parent
                .local_methods
                .values()
                .filter(|x| x.is_public())
                .map(|x| x.compile_for_trait_impl())
                .collect();

            compiled_traits.push(quote! {
                impl #parent_trait_ident for #ident {
                    #(#trait_declarations)*
                }
            });
            curr_parent_op = parent.parent.clone();
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

/// Note, does not check for deduplication
fn merge_generics(g1: &Generics, g2: &Generics) -> Generics {
    let mut output = g1.clone();
    output.params.extend(g2.params.clone());

    if let Some(w2) = g2.where_clause.clone() {
        match &mut output.where_clause {
            Some(w1) => {
                // Both have where clauses - extend predicates
                w1.predicates.extend(w2.predicates);
            }
            None => {
                output.where_clause = Some(w2);
            }
        }
    }
    output
}
