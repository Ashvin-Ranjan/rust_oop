// Truthfully, I got stuck so badly on this and ran out of motivation to do the edge cases that
// I just slopped this code out via Claude and looked over it, thats why it looks better than
// the rest of the codebase.

use std::collections::HashMap;

use syn::{
    AngleBracketedGenericArguments, Expr, GenericArgument, GenericParam, Path, PathArguments,
    PathSegment, ReturnType, Type, TypeArray, TypeGroup, TypeParen, TypePath, TypePtr,
    TypeReference, TypeSlice, TypeTuple,
};

pub fn generic_param_to_string(generic_param: &GenericParam) -> String {
    match generic_param {
        GenericParam::Const(const_param) => const_param.ident.to_string(),
        GenericParam::Lifetime(lifetime) => format!("'{}", lifetime.lifetime.ident),
        GenericParam::Type(type_param) => type_param.ident.to_string(),
    }
}

pub fn map_generic_arg(
    generic_arg: GenericArgument,
    mapping: &HashMap<String, GenericArgument>,
) -> GenericArgument {
    match &generic_arg {
        GenericArgument::Lifetime(lifetime) => {
            let lifetime_key = format!("'{}", lifetime.ident);
            if let Some(arg) = mapping.get(&lifetime_key) {
                return arg.clone();
            }
            generic_arg
        }
        GenericArgument::Type(type_param) => {
            GenericArgument::Type(map_type_with_generics(type_param, mapping))
        }
        GenericArgument::Const(expr) => {
            GenericArgument::Const(map_expression_with_generics(expr, mapping))
        }
        _ => generic_arg,
    }
}

pub fn map_type_with_generics(ty: &Type, mapping: &HashMap<String, GenericArgument>) -> Type {
    match ty {
        Type::Array(type_array) => Type::Array(TypeArray {
            bracket_token: type_array.bracket_token,
            elem: Box::new(map_type_with_generics(&type_array.elem, mapping)),
            semi_token: type_array.semi_token,
            len: map_expression_with_generics(&type_array.len, mapping),
        }),
        Type::BareFn(_) => ty.clone(),
        Type::Group(type_group) => Type::Group(TypeGroup {
            group_token: type_group.group_token,
            elem: Box::new(map_type_with_generics(&type_group.elem, mapping)),
        }),
        Type::ImplTrait(_) => ty.clone(),
        Type::Infer(_) => ty.clone(),
        Type::Macro(_) => ty.clone(),
        Type::Never(_) => ty.clone(),
        Type::Paren(type_paren) => Type::Paren(TypeParen {
            paren_token: type_paren.paren_token,
            elem: Box::new(map_type_with_generics(&type_paren.elem, mapping)),
        }),
        Type::Path(type_path) => {
            // Check if this is a bare type parameter (single-segment, no arguments)
            if type_path.qself.is_none() {
                let segs = &type_path.path.segments;
                if segs.len() == 1 && matches!(segs[0].arguments, PathArguments::None) {
                    let name = segs[0].ident.to_string();
                    if let Some(GenericArgument::Type(mapped_ty)) = mapping.get(&name) {
                        return mapped_ty.clone();
                    }
                }
            }
            // Otherwise recurse into any angle-bracketed path arguments (e.g. Vec<T>)
            Type::Path(TypePath {
                qself: type_path.qself.clone(),
                path: Path {
                    leading_colon: type_path.path.leading_colon,
                    segments: type_path
                        .path
                        .segments
                        .iter()
                        .map(|seg| PathSegment {
                            ident: seg.ident.clone(),
                            arguments: match &seg.arguments {
                                PathArguments::None => PathArguments::None,
                                PathArguments::AngleBracketed(ab) => {
                                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                        colon2_token: ab.colon2_token,
                                        lt_token: ab.lt_token,
                                        args: ab
                                            .args
                                            .iter()
                                            .map(|a| map_generic_arg(a.clone(), mapping))
                                            .collect(),
                                        gt_token: ab.gt_token,
                                    })
                                }
                                PathArguments::Parenthesized(p) => {
                                    PathArguments::Parenthesized(p.clone())
                                }
                            },
                        })
                        .collect(),
                },
            })
        }
        Type::Ptr(type_ptr) => Type::Ptr(TypePtr {
            star_token: type_ptr.star_token,
            const_token: type_ptr.const_token,
            mutability: type_ptr.mutability,
            elem: Box::new(map_type_with_generics(&type_ptr.elem, mapping)),
        }),
        Type::Reference(type_reference) => Type::Reference(TypeReference {
            and_token: type_reference.and_token,
            lifetime: type_reference.lifetime.clone(),
            mutability: type_reference.mutability,
            elem: Box::new(map_type_with_generics(&type_reference.elem, mapping)),
        }),
        Type::Slice(type_slice) => Type::Slice(TypeSlice {
            bracket_token: type_slice.bracket_token,
            elem: Box::new(map_type_with_generics(&type_slice.elem, mapping)),
        }),
        Type::TraitObject(_) => ty.clone(),
        Type::Tuple(type_tuple) => {
            let mapped_elems = type_tuple
                .elems
                .iter()
                .map(|x| map_type_with_generics(x, mapping))
                .collect();
            Type::Tuple(TypeTuple {
                paren_token: type_tuple.paren_token,
                elems: mapped_elems,
            })
        }
        Type::Verbatim(_) => ty.clone(),
        _ => ty.clone(),
    }
}

pub fn map_return_type_with_generics(
    rt: &ReturnType,
    mapping: &HashMap<String, GenericArgument>,
) -> ReturnType {
    match rt {
        ReturnType::Default => ReturnType::Default,
        ReturnType::Type(arrow, ty) => {
            ReturnType::Type(*arrow, Box::new(map_type_with_generics(ty, mapping)))
        }
    }
}

pub fn map_expression_with_generics(
    expr: &Expr,
    mapping: &HashMap<String, GenericArgument>,
) -> Expr {
    match expr {
        Expr::Lit(_) => expr.clone(),
        Expr::Path(ep)
            if ep.qself.is_none()
                && ep.path.segments.len() == 1
                && matches!(ep.path.segments[0].arguments, PathArguments::None) =>
        {
            let name = ep.path.segments[0].ident.to_string();
            if let Some(GenericArgument::Const(mapped_expr)) = mapping.get(&name) {
                mapped_expr.clone()
            } else {
                expr.clone()
            }
        }
        _ => expr.clone(),
    }
}
