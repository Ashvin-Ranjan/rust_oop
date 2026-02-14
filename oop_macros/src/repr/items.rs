use std::collections::HashSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Block, Expr, Generics, Ident, ReturnType, Token, Type, punctuated::Punctuated, spanned::Spanned,
};

use crate::{
    ast::items::{ClassConstructor, ClassField, ClassMethod, MethodArgument},
    repr::keywords::{CONSTRUCTOR_KW, SELF_KW},
};

#[derive(Debug)]
pub struct StaticClassFieldInformation {
    is_public: bool,
    ident: Ident,
    ty: Type,
    expression: Expr,
}

impl StaticClassFieldInformation {
    pub fn construct(field: ClassField) -> syn::Result<Self> {
        if field.static_kw.is_none() {
            panic!("Internal State Error: StaticClassFieldInformation called with static field")
        }
        let is_public = !field.pub_kw.is_none();
        let ident = field.ident;
        let ty = field.ty;
        if let Some(expression) = field.expression {
            return Ok(StaticClassFieldInformation {
                is_public,
                ident,
                ty,
                expression,
            });
        }
        Err(syn::Error::new(
            field.static_kw.span(),
            "Static fields must be initalized.",
        ))
    }

    pub fn compile(&self) -> TokenStream2 {
        let StaticClassFieldInformation {
            is_public,
            ident,
            ty,
            expression,
        } = self;

        let field_visibility = if *is_public {
            quote! {pub}
        } else {
            quote! {}
        };

        quote! {
            const #field_visibility #ident : #ty = #expression;
        }
    }
}

#[derive(Debug)]
pub struct LocalClassFieldInformation {
    is_public: bool,
    ident: Ident,
    ty: Type,
}

impl LocalClassFieldInformation {
    pub fn construct(field: ClassField) -> syn::Result<Self> {
        if !field.static_kw.is_none() {
            panic!("Internal State Error: LocalClassFieldInformation called with static field")
        }
        let is_public = !field.pub_kw.is_none();
        let ident = field.ident;
        let ty = field.ty;
        if !field.expression.is_none() {
            return Err(syn::Error::new(
                field.expression.span(),
                "Local fields cannot be assigned a value outside of methods.",
            ));
        }
        Ok(LocalClassFieldInformation {
            is_public,
            ident,
            ty,
        })
    }

    pub fn compile(&self, show_visibility: bool) -> TokenStream2 {
        let LocalClassFieldInformation {
            is_public,
            ident,
            ty,
        } = self;

        let field_visibility = if *is_public && show_visibility {
            quote! {pub}
        } else {
            quote! {}
        };

        quote! {
            #field_visibility #ident : #ty
        }
    }

    pub fn compile_as_names(&self) -> TokenStream2 {
        let LocalClassFieldInformation { ident, .. } = self;

        quote! {
            #ident
        }
    }
}

#[derive(Debug)]
pub struct StaticClassMethodInformation {
    is_public: bool,
    is_constructor: bool,
    ident: Ident,
    args: Vec<MethodArgumentInformation>,
    return_type: Option<ReturnType>,
    block: Block,
}

impl StaticClassMethodInformation {
    pub fn construct_from_method(method: ClassMethod) -> syn::Result<Self> {
        if method.static_kw.is_none() {
            panic!(
                "Internal State Error: StaticClassMethodInformation called with non-static field"
            )
        }
        if !method.const_kw.is_none() {
            return Err(syn::Error::new(
                method.const_kw.span(),
                "`const` cannot be applied to static functions.",
            ));
        }
        let is_public = !method.pub_kw.is_none();
        let ident = method.ident;
        let args = MethodArgumentInformation::construct_from_list(method.arguments, false)?;

        let return_type = Some(method.return_type);
        let block = method.block;
        if !method.generics.params.is_empty() {
            return Err(syn::Error::new(
                method.generics.span(),
                "Generics in class methods is currently not supported.",
            ));
        }
        Ok(StaticClassMethodInformation {
            is_public,
            is_constructor: false,
            ident,
            args,
            return_type,
            block,
        })
    }

    pub fn construct_from_constructor(
        constructor: ClassConstructor,
        class_name: &String,
    ) -> syn::Result<Self> {
        let is_public = !constructor.pub_kw.is_none();
        if constructor.ident.to_string() != *class_name {
            return Err(syn::Error::new(
                constructor.ident.span(),
                format!("Constructor must be named `{}`.", class_name),
            ));
        }
        let ident = format_ident!("{}", CONSTRUCTOR_KW);
        let args = MethodArgumentInformation::construct_from_list(constructor.arguments, false)?;
        let block = constructor.block;

        Ok(StaticClassMethodInformation {
            is_public,
            is_constructor: true,
            ident,
            args,
            return_type: None,
            block,
        })
    }

    pub fn compile(&self, generics: &Generics, class_ident: &Ident) -> TokenStream2 {
        let StaticClassMethodInformation {
            is_public,
            is_constructor,
            ident,
            args,
            return_type,
            block,
        } = self;

        let field_visibility = if *is_public {
            quote! {pub}
        } else {
            quote! {}
        };

        let where_clause = &generics.where_clause;

        let args_compiled: Vec<TokenStream2> = args.iter().map(|x| x.compile()).collect();

        let mut ret_val = quote! { #return_type };
        if *is_constructor {
            if !return_type.is_none() {
                panic!("Internal State Error: `return_type` is not `None` in constructor");
            }
            ret_val = quote! { -> #class_ident }
        } else if return_type.is_none() {
            panic!(
                "Internal State Error: `return_type` is `None` in non-constructor static method."
            );
        }

        quote! {
            #field_visibility fn #ident #generics (#(#args_compiled ,)*) #ret_val #where_clause #block
        }
    }
}

#[derive(Debug)]
pub struct LocalClassMethodInformation {
    is_public: bool,
    is_constant: bool,
    ident: Ident,
    args: Vec<MethodArgumentInformation>,
    return_type: ReturnType,
    block: Block,
}

impl LocalClassMethodInformation {
    pub fn construct(method: ClassMethod) -> syn::Result<Self> {
        if !method.static_kw.is_none() {
            panic!("Internal State Error: LocalClassMethodInformation called with static field")
        }
        let is_public = !method.pub_kw.is_none();
        let is_constant = !method.const_kw.is_none();
        let ident = method.ident;
        let args = MethodArgumentInformation::construct_from_list(method.arguments, true)?;

        let return_type = method.return_type;
        let block = method.block;
        if !method.generics.params.is_empty() {
            return Err(syn::Error::new(
                method.generics.span(),
                "Generics in class methods is currently not supported.",
            ));
        }
        Ok(LocalClassMethodInformation {
            is_public,
            is_constant,
            ident,
            args,
            return_type,
            block,
        })
    }

    pub fn compile(&self) -> TokenStream2 {
        let LocalClassMethodInformation {
            is_public,
            is_constant,
            ident,
            args,
            return_type,
            block,
        } = self;

        let field_visibility = if *is_public {
            quote! {pub}
        } else {
            quote! {}
        };

        let reciever = if *is_constant {
            quote! {&self}
        } else {
            quote! {&mut self}
        };

        let args_compiled: Vec<TokenStream2> = args.iter().map(|x| x.compile()).collect();

        quote! {
            #field_visibility fn #ident (#reciever, #(#args_compiled ,)*) #return_type #block
        }
    }
}

#[derive(Debug)]
struct MethodArgumentInformation {
    ident: Ident,
    ty: Type,
}

impl MethodArgumentInformation {
    pub fn construct(method_arg: MethodArgument) -> syn::Result<MethodArgumentInformation> {
        let ident = method_arg.ident;
        let ty = method_arg.ty;
        Ok(MethodArgumentInformation { ident, ty })
    }

    pub fn compile(&self) -> TokenStream2 {
        let MethodArgumentInformation { ident, ty } = self;
        quote! {
            #ident : #ty
        }
    }

    pub fn construct_from_list(
        raw_args: Punctuated<MethodArgument, Token![,]>,
        self_reserved: bool,
    ) -> syn::Result<Vec<MethodArgumentInformation>> {
        let args = raw_args
            .into_iter()
            .map(|x| MethodArgumentInformation::construct(x))
            .collect::<syn::Result<Vec<MethodArgumentInformation>>>()?;

        let mut arg_names = HashSet::new();
        for arg in &args {
            if self_reserved && arg.ident.to_string() == SELF_KW {
                return Err(syn::Error::new(
                    arg.ident.span(),
                    format!("`{}` is a resreved keyword in local functions.", SELF_KW),
                ));
            }
            if arg_names.contains(&arg.ident.to_string()) {
                return Err(syn::Error::new(
                    arg.ident.span(),
                    format!("Duplicate definition of argument `{}`.", arg.ident),
                ));
            } else {
                arg_names.insert(arg.ident.to_string());
            }
        }
        Ok(args)
    }
}
