use syn::{
    AngleBracketedGenericArguments, Generics, Ident, Token, braced,
    parse::{Parse, ParseStream},
    token::Brace,
};

use crate::ast::{items::ClassItem, utils::parse_until_empty};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(class);
}

#[derive(Debug)]
pub struct MacroBlock {
    pub definitions: Vec<ClassDefinition>,
}

impl Parse for MacroBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MacroBlock {
            definitions: parse_until_empty::<ClassDefinition>(input)?,
        })
    }
}

/// Syntax for ClassDefinition is as follows:
/// (pub)? class syn::Ident syn::Generics : (syn::Ident (+ syn::Ident)*)? where_clause { (ClassField)* }
/// TODO:
/// - Attribute Handling
#[derive(Debug)]
pub struct ClassDefinition {
    pub pub_kw: Option<Token![pub]>,
    pub _class_kw: kw::class,
    pub class_ident: Ident,
    /// This includes the where clause
    pub generics: Generics,
    pub _colon: Option<Token![:]>,
    pub parent: Option<Ident>,
    pub parent_generics: Option<AngleBracketedGenericArguments>,
    pub _braces: Brace,
    pub items: Vec<ClassItem>,
}

impl Parse for ClassDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pub_kw = input.parse::<Option<Token![pub]>>()?;
        let class_kw = input.parse::<kw::class>()?;
        let class_ident = input.parse::<Ident>()?;
        let mut generics = input.call(Generics::parse)?;
        let colon = input.parse::<Option<Token![:]>>()?;
        let mut parent = None;
        let mut parent_generics = None;
        if colon.is_some() {
            parent = Some(input.parse::<Ident>()?);
            if input.peek(Token![<]) {
                parent_generics = Some(input.parse::<AngleBracketedGenericArguments>()?);
            }
        }
        generics.where_clause = input.parse()?;

        let content;
        let braces = braced!(content in input);
        let items = parse_until_empty::<ClassItem>(&content)?;

        Ok(ClassDefinition {
            pub_kw,
            _class_kw: class_kw,
            class_ident,
            generics,
            _colon: colon,
            parent,
            parent_generics,
            _braces: braces,
            items,
        })
    }
}
