use syn::parse::{Parse, ParseStream};

pub fn parse_until_empty<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let mut result = Vec::new();
    while !input.is_empty() {
        result.push(input.parse()?);
    }
    Ok(result)
}
