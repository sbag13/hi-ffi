use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::{LazyLock, Mutex};

use impl_block_wrapper::ImplBlockWrapper;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

static ENUM_TYPES: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));
static STRUCT_TYPES: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

pub fn register_enum_type(name: String) {
    ENUM_TYPES.lock().unwrap().insert(name);
}

pub fn is_enum_type(name: &str) -> bool {
    ENUM_TYPES.lock().unwrap().contains(name)
}

pub fn is_struct_type(name: &str) -> bool {
    STRUCT_TYPES.lock().unwrap().contains(name)
}

pub fn register_struct_type(name: String) {
    STRUCT_TYPES.lock().unwrap().insert(name);
}

pub mod base;
#[cfg(feature = "cpp")]
pub mod cpp;
pub mod enum_wrapper;
pub mod function_wrapper;
pub mod impl_block_wrapper;
pub mod struct_wrapper;
#[cfg(feature = "swift")]
pub mod swift;
use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::wrapper::enum_wrapper::EnumWrapper;
pub use function_wrapper::*;
pub use struct_wrapper::*;
#[cfg(feature = "python")]
pub mod python;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ReusableWrapper {
    Vec(WrapperType),
    Result(WrapperType),
}

impl From<&ReusableWrapper> for TokenStream2 {
    fn from(wrapper: &ReusableWrapper) -> TokenStream2 {
        match wrapper {
            ReusableWrapper::Vec(inner) => generate_vec_wrapper(inner),
            ReusableWrapper::Result(inner) => generate_result_wrapper(inner),
        }
    }
}

fn generate_result_wrapper(inner: &WrapperType) -> TokenStream2 {
    let drop_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{}_result", inner.name());
    let wrapper_fn_name_drop = match inner {
        WrapperType::Vec(vec_inner) => format_ident!("drop_{}_vec_result", vec_inner.name()),
        _ => format_ident!("drop_{}_result", inner.name()),
    };

    let unwrap_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{}_result", inner.name());
    let wrapper_fn_name_unwrap = format_ident!("unwrap_{}_result", inner.name());

    let unwrap_err_ext_fn_name = format!(
        "{EXPORTED_SYMBOLS_PREFIX}__unwrap_err_{}_result",
        inner.name()
    );
    let wrapper_fn_name_unwrap_err = format_ident!("unwrap_err_{}_result", inner.name());

    let is_err_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_err_{}_result", inner.name());
    let wrapper_fn_name_is_err = format_ident!("is_err_{}_result", inner.name());

    let inner_type: TokenStream2 = match inner {
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name: TokenStream2 = vec_inner.name().parse().unwrap();
            quote! {Vec<#vec_inner_name>}
        }
        WrapperType::UnitExpr => quote! {()},
        _ => inner.name().parse().unwrap(),
    };

    let unwrap_clone_expr = match &inner {
        WrapperType::String | WrapperType::Struct(_) | WrapperType::Vec(_) => quote! {
            Box::into_raw(Box::new(value.clone()))
        },
        WrapperType::Bool
        | WrapperType::Enum(_)
        | WrapperType::FloatingPointNumber(_)
        | WrapperType::IntegerNumber(_) => quote! {
            value.clone()
        },
        WrapperType::Result(_) => {
            panic!("Nested Result types are not supported")
        }
        WrapperType::UnitExpr => quote! {()},
    };

    let clone_ret_type: TokenStream2 = match &inner {
        WrapperType::String => "*mut std::string::String".parse().unwrap(),
        WrapperType::Struct(name) => format!("*mut {name}").parse().unwrap(),
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name = vec_inner.name();
            format!("*mut Vec<{}>", vec_inner_name).parse().unwrap()
        }
        WrapperType::UnitExpr => quote! {()},
        _ => inner.name().parse().unwrap(),
    };

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #drop_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_drop(_self: *mut std::result::Result<#inner_type, std::sync::Arc<dyn std::error::Error>>) {
            unsafe {
                if !_self.is_null() {
                    let _ = Box::from_raw(_self);
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #unwrap_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_unwrap(_self: *mut std::result::Result<#inner_type, std::sync::Arc<dyn std::error::Error>>) -> #clone_ret_type {
            unsafe {
                match &* _self {
                    Ok(value) => {
                        #unwrap_clone_expr
                    },
                    Err(_) => panic!("Called unwrap on an Err value"),
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #unwrap_err_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_unwrap_err(_self: *mut std::result::Result<#inner_type, std::sync::Arc<dyn std::error::Error>>) -> *mut std::sync::Arc<dyn std::error::Error> {
            unsafe {
                match &* _self {
                    Ok(_) => panic!("Called unwrap_err on an Ok value"),
                    Err(err) => {
                        Box::into_raw(Box::new((*err).clone()))
                    },
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #is_err_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_is_err(_self: *mut std::result::Result<#inner_type, std::sync::Arc<dyn std::error::Error>>) -> bool {
            unsafe {
                match &* _self {
                    Ok(_) => false,
                    Err(_) => true,
                }
            }
        }
    }
}

fn generate_vec_wrapper(inner: &WrapperType) -> TokenStream2 {
    let drop_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{}_vec", inner.name());
    let wrapper_fn_name_drop = format_ident!("drop_{}_vec", inner.name());

    let with_capacity_ext_fn_name = format!(
        "{EXPORTED_SYMBOLS_PREFIX}__with_capacity_{}_vec",
        inner.name()
    );
    let wrapper_fn_name_with_capacity = format_ident!("with_capacity_{}_vec", inner.name());

    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__push_{}_vec", inner.name());
    let wrapper_fn_name_push = format_ident!("push_{}_vec", inner.name());

    // New externs for reading returned vectors
    let len_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__len_{}_vec", inner.name());
    let wrapper_fn_name_len = format_ident!("len_{}_vec", inner.name());

    let get_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__get_{}_vec", inner.name());
    let wrapper_fn_name_get = format_ident!("get_{}_vec", inner.name());

    let value_receiver: TokenStream2 = match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            format!("value: {inner}").parse().unwrap()
        }
        WrapperType::Bool => "value: bool".parse().unwrap(),
        WrapperType::String => "ptr: *const i8, _len: usize".parse().unwrap(),
        WrapperType::Struct(name) => format!("value: *mut {name}").parse().unwrap(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported yet!"),
        WrapperType::Result(_) => {
            panic!("Vec of Result type not supported yet!")
        }
        WrapperType::Enum(name) => format!("value: {name}").parse().unwrap(),
        WrapperType::UnitExpr => panic!("UnitExpr as vec inner type is not supported"),
    };
    let value_cast = match inner {
        WrapperType::String => quote! {
            let value = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
        },
        WrapperType::Struct(_) => {
            quote! {
                let value = unsafe { (*value).clone() };
            }
        }
        _ => quote! {},
    };

    let vec_type: TokenStream2 = format!("Vec<{}>", inner.name()).parse().unwrap();

    // Return type for get() depending on inner type
    let get_return_type: TokenStream2 = match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            inner.parse().unwrap()
        }
        WrapperType::Bool => "bool".parse().unwrap(),
        WrapperType::String => "*mut std::string::String".parse().unwrap(),
        WrapperType::Struct(name) => format!("*mut {name}").parse().unwrap(),
        WrapperType::Vec(_) => panic!("Vec of vecs not supported yet!"),
        WrapperType::Enum(name) => name.parse().unwrap(),
        WrapperType::Result(_) => {
            panic!("Vec of Result type not supported yet!")
        }
        WrapperType::UnitExpr => panic!("UnitExpr not supported yet as get return type!"),
    };

    let get_body: TokenStream2 = match inner {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool => {
            quote! {
                (&*_self)[index]
            }
        }
        WrapperType::String | WrapperType::Struct(_) => quote! {{
            let v = (&*_self)[index].clone();
            Box::into_raw(Box::new(v))
        }},
        WrapperType::Vec(_) => unreachable!(),
        WrapperType::Enum(_) => quote! {
            (&*_self)[index]
        },
        WrapperType::Result(_) => {
            panic!("Vec of Result type not supported yet!")
        }
        WrapperType::UnitExpr => {
            panic!("UnitExpr not supported yet as get return type!")
        }
    };

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #drop_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_drop(_self: *mut #vec_type) {
            unsafe {
                if !_self.is_null() {
                    let _ = Box::from_raw(_self);
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #with_capacity_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_with_capacity(capacity: usize) -> *mut #vec_type {
            unsafe {
                let vec = Box::new(Vec::with_capacity(capacity));
                Box::into_raw(vec)
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #push_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_push(_self: *mut #vec_type, #value_receiver) {
            #value_cast
            (&mut *_self).push(value);
        }

        // New len extern for reading vectors
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #len_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_len(_self: *mut #vec_type) -> usize {
            (&*_self).len()
        }

        // New get extern for reading vectors
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #get_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_get(_self: *mut #vec_type, index: usize) -> #get_return_type {
            #get_body
        }
    }
}

#[derive(Debug)]
pub struct Wrapper {
    pub(crate) original_definition: TokenStream2,
    pub(crate) parsed: ParsedWrapper,
    pub(crate) reusable_wrappers: HashSet<ReusableWrapper>,
}

impl Wrapper {
    pub fn name(&self) -> String {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => struct_wrapper.name.to_string(),
            ParsedWrapper::Function(function_wrapper) => function_wrapper.name.to_string(),
            ParsedWrapper::ImplBlock(impl_block_wrapper) => {
                impl_block_wrapper.struct_name.to_string()
            }
            ParsedWrapper::Enum(enum_wrapper) => enum_wrapper.name.to_string(),
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
                ..
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
                ..
            } => {
                let tokens: TokenStream2 = impl_block_wrapper.into();
                quote! {
                    #original_definition
                    #tokens
                }
            }
            Wrapper {
                parsed: ParsedWrapper::Enum(enum_wrapper),
                original_definition,
                ..
            } => {
                let tokens: TokenStream2 = enum_wrapper.into();
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
    Enum(EnumWrapper),
}

impl From<Wrapper> for TokenStream {
    fn from(wrapper: Wrapper) -> TokenStream {
        let token_stream: TokenStream2 = (&wrapper).into();
        token_stream.into()
    }
}
