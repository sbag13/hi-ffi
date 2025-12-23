use std::collections::HashSet;
use std::fmt::Debug;

use impl_block_wrapper::ImplBlockWrapper;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

pub mod base;
#[cfg(feature = "cpp")]
pub mod cpp;
pub mod function_wrapper;
pub mod impl_block_wrapper;
pub mod struct_wrapper;
#[cfg(feature = "swift")]
pub mod swift;
use crate::EXPORTED_SYMBOLS_PREFIX;
pub use function_wrapper::*;
pub use struct_wrapper::*;
#[cfg(feature = "python")]
pub mod python;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ReusableWrapper {
    Vec(WrapperType),
}

impl From<&ReusableWrapper> for TokenStream2 {
    fn from(wrapper: &ReusableWrapper) -> TokenStream2 {
        match wrapper {
            ReusableWrapper::Vec(inner) => generate_vec_wrapper(inner),
        }
    }
}

fn generate_vec_wrapper(inner: &WrapperType) -> TokenStream2 {
    let drop_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{}_vec", inner.name());
    let wrapper_fn_name = format_ident!("drop_{}_vec", inner.name());

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
        WrapperType::Enum(name) => format!("value: {name}").parse().unwrap(),
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
    };

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #drop_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #vec_type) {
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
