use std::fmt::Debug;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub mod base;
#[cfg(feature = "cpp")]
mod cpp;
pub mod function_wrapper;
pub mod struct_wrapper;
#[cfg(feature = "swift")]
mod swift;

#[cfg(feature = "cpp")]
pub use cpp::*;
pub use function_wrapper::*;
pub use struct_wrapper::*;
#[cfg(feature = "swift")]
pub use swift::*;

#[derive(Debug)]
pub struct Wrapper {
    pub(crate) original_definition: TokenStream2,
    pub(crate) parsed: ParsedWrapper,
}

impl Wrapper {
    pub fn name(&self) -> String {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => struct_wrapper.name.to_string(),
            ParsedWrapper::Function(function_wrapper) => function_wrapper.name.to_string(),
        }
    }
}

impl From<&Wrapper> for TokenStream2 {
    fn from(wrapper: &Wrapper) -> TokenStream2 {
        match wrapper {
            Wrapper {
                parsed: ParsedWrapper::Struct(struct_wrapper),
                ..
            } => struct_wrapper.into(),
            Wrapper {
                parsed: ParsedWrapper::Function(function_wrapper),
                original_definition,
            } => {
                let tokens: TokenStream2 = function_wrapper.into();
                quote! {
                    #original_definition
                    #tokens
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ParsedWrapper {
    Struct(StructWrapper),
    Function(FunctionWrapper),
}

impl From<Wrapper> for TokenStream {
    fn from(wrapper: Wrapper) -> TokenStream {
        let token_stream: TokenStream2 = (&wrapper).into();
        token_stream.into()
    }
}
