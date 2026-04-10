use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::fmt::Debug;
use syn::Ident;

pub struct EnumWrapper {
    pub(crate) name: Ident,
    pub(crate) variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub struct EnumVariant {
    #[allow(dead_code)]
    pub(crate) name: Ident,
    #[allow(dead_code)]
    pub(crate) discriminant: Option<String>,
}

impl Debug for EnumWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumWrapper")
            .field("name", &self.name)
            .field("variants", &self.variants)
            .finish()
    }
}

impl From<&EnumWrapper> for TokenStream2 {
    fn from(_enum_wrapper: &EnumWrapper) -> TokenStream2 {
        // For enums, we don't need to generate any additional wrapper functions
        // since C-like enums are directly compatible with C
        quote! {}
    }
}
