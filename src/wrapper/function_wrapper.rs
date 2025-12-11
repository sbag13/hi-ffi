use std::fmt::Debug;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

#[derive(Debug)]
pub struct FunctionWrapper {
    pub(crate) name: syn::Ident,
    pub(crate) extern_function_name: String,
    pub(crate) args_wrappers: Vec<FunctionArgWrapper>,
    pub(crate) return_wrapper: Option<FunctionReturnWrapper>,
}

pub struct MappedReturnType {
    pub return_type_sig: TokenStream2,
    pub result_cast: TokenStream2,
}

pub fn map_return_type(return_wrapper: &Option<FunctionReturnWrapper>) -> MappedReturnType {
    match return_wrapper {
        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::Primitive,
            return_type,
        }) => MappedReturnType {
            return_type_sig: quote! {-> #return_type},
            result_cast: quote! {result},
        },
        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::String,
            ..
        }) => MappedReturnType {
            return_type_sig: quote! {-> *mut String},
            result_cast: quote! {
                Box::into_raw(Box::new(result))
            },
        },
        Some(FunctionReturnWrapper {
            wrapper_type: FunctionReturnWrapperType::Struct,
            return_type,
        }) => MappedReturnType {
            return_type_sig: quote! {-> *mut #return_type},
            result_cast: quote! {
                Box::into_raw(Box::new(result))
            },
        },
        None => MappedReturnType {
            return_type_sig: quote! {},
            result_cast: quote! {result},
        },
    }
}

impl From<&FunctionWrapper> for TokenStream2 {
    fn from(function_wrapper: &FunctionWrapper) -> Self {
        let fn_name = &function_wrapper.name;
        let extern_function_name = &function_wrapper.extern_function_name;
        let wrapper_name = format_ident!("ffi_wrapper_{}", fn_name);

        let MappedFunctionArgsTokens {
            arg_signatures,
            arg_names,
            arg_casts,
        } = map_function_arg_wrappers(function_wrapper.args_wrappers.iter());

        let MappedReturnType {
            return_type_sig,
            result_cast,
        } = map_return_type(&function_wrapper.return_wrapper);

        quote! {
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            #[unsafe(export_name = #extern_function_name)]
            pub unsafe extern "C" fn #wrapper_name(#(#arg_signatures,)*) #return_type_sig {
                #(#arg_casts)*
                let result = #fn_name(#(#arg_names,)*);
                #result_cast
            }
        }
    }
}

pub struct MappedFunctionArgsTokens {
    pub arg_signatures: Vec<TokenStream2>,
    pub arg_names: Vec<TokenStream2>,
    pub arg_casts: Vec<TokenStream2>,
}
pub fn map_function_arg_wrappers<'a>(
    args: impl Iterator<Item = &'a FunctionArgWrapper>,
) -> MappedFunctionArgsTokens {
    let (mut arg_signatures, mut arg_names, mut arg_casts): (Vec<_>, Vec<_>, Vec<_>) =
        (Vec::new(), Vec::new(), Vec::new());
    args
    .for_each(|arg| match arg {
        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: FunctionArgWrapperType::Primitive,
        } => {
            arg_signatures.push(quote! {#arg_name: #arg_type});
            arg_names.push(quote! {#arg_name});
        }
        FunctionArgWrapper {
            arg_name,
            wrapper_type: FunctionArgWrapperType::String,
            ..
        } => {
            arg_signatures.push(quote! {#arg_name: *const i8});
            arg_names.push(quote! {#arg_name});
            arg_casts.push(quote! {
                let #arg_name = unsafe { std::ffi::CStr::from_ptr(#arg_name).to_str().unwrap().to_owned() };
            });
        }
        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: FunctionArgWrapperType::Struct,
        } => {
            arg_signatures.push(quote! {#arg_name: *mut #arg_type});
            arg_names.push(quote! {#arg_name});
            arg_casts.push(quote! {
                let #arg_name = unsafe { (*#arg_name).clone() };
            });
        }
    });
    MappedFunctionArgsTokens {
        arg_signatures,
        arg_names,
        arg_casts,
    }
}

pub struct FunctionArgWrapper {
    pub(crate) wrapper_type: FunctionArgWrapperType,
    pub(crate) arg_name: syn::Ident,
    pub(crate) arg_type: syn::Type,
}

impl Debug for FunctionArgWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionArgWrapper")
            .field("arg_name", &self.arg_name)
            .finish()
    }
}

#[derive(Debug)]
pub enum FunctionArgWrapperType {
    Primitive,
    String,
    Struct,
}

pub struct FunctionReturnWrapper {
    pub(crate) wrapper_type: FunctionReturnWrapperType,
    pub(crate) return_type: syn::Type,
}

impl Debug for FunctionReturnWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionReturnWrapper")
            .field("wrapper_type", &self.wrapper_type)
            .finish()
    }
}

#[derive(Debug)]
pub enum FunctionReturnWrapperType {
    Primitive,
    String,
    Struct,
}
