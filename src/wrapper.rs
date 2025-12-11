use std::fmt::Debug;

use impl_block_wrapper::ImplBlockWrapper;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub mod base;
#[cfg(feature = "cpp")]
pub mod cpp;
pub mod function_wrapper;
pub mod impl_block_wrapper;
pub mod struct_wrapper;
#[cfg(feature = "swift")]
pub mod swift;
pub use function_wrapper::*;
pub use struct_wrapper::*;
#[cfg(feature = "python")]
pub mod python;

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
            ParsedWrapper::ImplBlock(impl_block_wrapper) => {
                impl_block_wrapper.struct_name.to_string()
            }
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
            Wrapper {
                parsed: ParsedWrapper::ImplBlock(impl_block_wrapper),
                original_definition,
            } => {
                let tokens: TokenStream2 = impl_block_wrapper.into();
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
    ImplBlock(ImplBlockWrapper),
}

impl From<Wrapper> for TokenStream {
    fn from(wrapper: Wrapper) -> TokenStream {
        let token_stream: TokenStream2 = (&wrapper).into();
        token_stream.into()
    }
}
