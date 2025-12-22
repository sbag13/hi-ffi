use std::fmt::Debug;
use std::str::FromStr;
use std::sync::{LazyLock, Mutex};

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

static ENUM_TYPES: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

pub fn register_enum_type(name: String) {
    ENUM_TYPES.lock().unwrap().insert(name);
}

pub fn is_enum_type(name: &str) -> bool {
    ENUM_TYPES.lock().unwrap().contains(name)
}

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
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_),
            return_type,
        }) => MappedReturnType {
            return_type_sig: quote! {-> #return_type},
            result_cast: quote! {result},
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::String,
            ..
        }) => MappedReturnType {
            return_type_sig: quote! {-> *mut String},
            result_cast: quote! {
                Box::into_raw(Box::new(result))
            },
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Struct(_),
            return_type,
        }) => MappedReturnType {
            return_type_sig: quote! {-> *mut #return_type},
            result_cast: quote! {
                Box::into_raw(Box::new(result))
            },
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Vec(_),
            return_type,
        }) => MappedReturnType {
            return_type_sig: quote! {-> *mut #return_type},
            result_cast: quote! {
                Box::into_raw(Box::new(result))
            },
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Enum(_),
            return_type,
        }) => MappedReturnType {
            return_type_sig: quote! {-> #return_type},
            result_cast: quote! {result},
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
            wrapper_type: WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_),
        } => {
            arg_signatures.push(quote! {#arg_name: #arg_type});
            arg_names.push(quote! {#arg_name});
        }
        FunctionArgWrapper {
            arg_name,
            wrapper_type: WrapperType::String,
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
            wrapper_type: WrapperType::Struct(_),
        } => {
            arg_signatures.push(quote! {#arg_name: *mut #arg_type});
            arg_names.push(quote! {#arg_name});
            arg_casts.push(quote! {
                let #arg_name = unsafe { (*#arg_name).clone() };
            });
        }
        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: WrapperType::Vec(_),
        } => {
            arg_signatures.push(quote! {#arg_name: *mut #arg_type});
            arg_names.push(quote! {new_vec});
            arg_casts.push(quote! {
                let mut new_vec = Vec::new();
                std::mem::swap(&mut new_vec, unsafe { &mut(*#arg_name)} );
            });
        }
        FunctionArgWrapper {
            arg_name,
            arg_type,
            wrapper_type: WrapperType::Enum(_),
        } => {
            arg_signatures.push(quote! {#arg_name: #arg_type});
            arg_names.push(quote! {#arg_name});
        }
    });
    MappedFunctionArgsTokens {
        arg_signatures,
        arg_names,
        arg_casts,
    }
}

pub struct FunctionArgWrapper {
    pub(crate) wrapper_type: WrapperType,
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum WrapperType {
    IntegerNumber(String),
    FloatingPointNumber(String),
    Bool,
    String,
    Struct(String),
    Vec(Box<WrapperType>),
    Enum(String),
}

impl WrapperType {
    pub fn name(&self) -> String {
        match self {
            WrapperType::IntegerNumber(inner)
            | WrapperType::FloatingPointNumber(inner)
            | WrapperType::Struct(inner)
            | WrapperType::Enum(inner) => inner.to_owned(),
            WrapperType::Vec(inner) => format!("vec_of_{}", inner.name()),
            WrapperType::Bool => "bool".to_string(),
            WrapperType::String => "String".to_string(),
        }
    }
}

impl FromStr for WrapperType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "usize" => Ok(WrapperType::IntegerNumber(s.to_string())),
            "f32" | "f64" => Ok(WrapperType::FloatingPointNumber(s.to_string())),
            "bool" => Ok(WrapperType::Bool),
            "String" => Ok(WrapperType::String),
            "str" => Err("str wrapper not supported".to_string()),
            _ => {
                if is_enum_type(s) {
                    Ok(WrapperType::Enum(s.to_string()))
                } else {
                    Ok(WrapperType::Struct(s.to_string()))
                }
            }
        }
    }
}

pub struct FunctionReturnWrapper {
    pub(crate) wrapper_type: WrapperType,
    pub(crate) return_type: syn::Type,
}

impl Debug for FunctionReturnWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionReturnWrapper")
            .field("wrapper_type", &self.wrapper_type)
            .finish()
    }
}
