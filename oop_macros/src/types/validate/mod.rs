pub trait Validate {
    fn validate(&self) -> syn::Result<()>
    where
        Self: Sized;
}
