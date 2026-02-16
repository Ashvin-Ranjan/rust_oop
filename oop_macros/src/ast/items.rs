use syn::{
    Block, Expr, Generics, Ident, ReturnType, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Paren,
};

#[derive(Debug)]
pub enum ClassItem {
    ClassField(ClassField),
    ClassMethod(ClassMethod),
    ClassConstructor(ClassConstructor),
}

impl Parse for ClassItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![let]) {
            Ok(ClassItem::ClassField(input.parse()?))
        } else {
            // Need to distinguish between method and constructor
            // Use fork to peek ahead
            let fork = input.fork();

            // Skip visibility & modifiers
            fork.parse::<Option<Token![pub]>>()?;

            let fork_lookahead = fork.lookahead1();

            if fork_lookahead.peek(Token![const])
                || fork_lookahead.peek(Token![static])
                || fork_lookahead.peek(Token![fn])
            {
                Ok(ClassItem::ClassMethod(input.parse()?))
            } else {
                Ok(ClassItem::ClassConstructor(input.parse()?))
            }
        }
    }
}

/// Syntax for ClassField is as follows:
/// let (pub)? (static)? ident : syn::Type (= syn::Expression)? ;
#[derive(Debug)]
pub struct ClassField {
    pub _let_kw: Token![let],
    pub pub_kw: Option<Token![pub]>,
    pub static_kw: Option<Token![static]>,
    pub ident: Ident,
    pub _colon: Token![:],
    pub ty: Type,
    pub _equals: Option<Token![=]>,
    pub expression: Option<Expr>,
    pub _semicolon: Token![;],
}

impl Parse for ClassField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let let_kw = input.parse::<Token![let]>()?;
        let pub_kw = input.parse::<Option<Token![pub]>>()?;
        let static_kw = input.parse::<Option<Token![static]>>()?;
        let ident = input.parse::<Ident>()?;
        let colon = input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        let equals = input.parse::<Option<Token![=]>>()?;
        let mut expression: Option<Expr> = None;
        if !equals.is_none() {
            expression = Some(input.parse()?);
        }
        let semicolon = input.parse::<Token![;]>()?;
        Ok(ClassField {
            _let_kw: let_kw,
            pub_kw,
            static_kw,
            ident,
            _colon: colon,
            ty,
            _equals: equals,
            expression,
            _semicolon: semicolon,
        })
    }
}

/// Syntax for ClassMethods is as follows:
/// (pub)? (static)? (const)? fn ident syn::Generics (Puntuated<MethodArguments, ,>) syn::ReturnType where_block syn::Block
#[derive(Debug)]
pub struct ClassMethod {
    pub pub_kw: Option<Token![pub]>,
    pub static_kw: Option<Token![static]>,
    pub const_kw: Option<Token![const]>,
    pub _fn_kw: Token![fn],
    pub ident: Ident,
    pub generics: Generics,
    pub _parenthesis: Paren,
    pub arguments: Punctuated<MethodArgument, Token![,]>,
    pub return_type: ReturnType,
    pub block: Block,
}

impl Parse for ClassMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pub_kw = input.parse::<Option<Token![pub]>>()?;
        let static_kw = input.parse::<Option<Token![static]>>()?;
        let const_kw = input.parse::<Option<Token![const]>>()?;
        let fn_kw = input.parse::<Token![fn]>()?;
        let ident = input.parse::<Ident>()?;
        let mut generics = input.parse::<Generics>()?;
        let content;
        let parenthesis = parenthesized!(content in input);
        let arguments = content.call(Punctuated::<MethodArgument, Token![,]>::parse_terminated)?;
        let return_type = input.parse::<ReturnType>()?;
        generics.where_clause = input.parse()?;
        let block = input.parse::<Block>()?;
        Ok(ClassMethod {
            pub_kw,
            static_kw,
            const_kw,
            _fn_kw: fn_kw,
            ident,
            generics,
            _parenthesis: parenthesis,
            arguments,
            return_type,
            block,
        })
    }
}

/// Syntax for ClassConstructor is as follows:
/// (pub)? ident :: ident syn::Generics (Puntuated<MethodArgument, ,>) where_block syn::Block
#[derive(Debug)]
pub struct ClassConstructor {
    pub pub_kw: Option<Token![pub]>,
    pub ident: Ident,
    pub generics: Generics,
    pub _parenthesis: Paren,
    pub arguments: Punctuated<MethodArgument, Token![,]>,
    pub block: Block,
}

impl Parse for ClassConstructor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pub_kw = input.parse::<Option<Token![pub]>>()?;
        let ident = input.parse::<Ident>()?;
        let mut generics = input.parse::<Generics>()?;
        let content;
        let parenthesis = parenthesized!(content in input);
        let arguments = content.call(Punctuated::<MethodArgument, Token![,]>::parse_terminated)?;
        generics.where_clause = input.parse()?;
        let block = input.parse::<Block>()?;
        Ok(ClassConstructor {
            pub_kw,
            ident,
            generics,
            _parenthesis: parenthesis,
            arguments,
            block,
        })
    }
}

/// Syntax for MethodArgument is as so:
/// Syn::Ident : Syn::Type
#[derive(Debug)]
pub struct MethodArgument {
    pub ident: Ident,
    pub _colon: Token![:],
    pub ty: Type,
}

impl Parse for MethodArgument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let colon = input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        Ok(MethodArgument {
            ident,
            _colon: colon,
            ty,
        })
    }
}
