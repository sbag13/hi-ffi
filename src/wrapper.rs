use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::Deref;
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
#[cfg(feature = "java")]
pub mod java;
pub mod struct_wrapper;
#[cfg(feature = "swift")]
pub mod swift;
pub mod trait_wrapper;
use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::wrapper::enum_wrapper::EnumWrapper;
use crate::wrapper::trait_wrapper::TraitWrapper;
pub use function_wrapper::*;
pub use struct_wrapper::*;
#[cfg(feature = "python")]
pub mod python;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ReusableWrapper {
    Vec(WrapperType),
    Result(WrapperType),
    Option(WrapperType),
    HashMap(WrapperType, WrapperType),
}

pub(crate) fn rust_type(wrapper: &WrapperType) -> TokenStream2 {
    match wrapper {
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name: TokenStream2 = rust_type(vec_inner.deref());
            quote! {Vec<#vec_inner_name>}
        }
        WrapperType::HashMap(key, value) => {
            let key_type = rust_type(key);
            let value_type = rust_type(value);
            quote! {std::collections::HashMap<#key_type, #value_type>}
        }
        WrapperType::Result(result_inner) => {
            let result_inner_name = rust_type(result_inner.deref());
            quote! {std::result::Result<#result_inner_name, std::sync::Arc<dyn std::error::Error>>}
        }
        WrapperType::Option(option_inner) => {
            let option_inner_name: TokenStream2 = rust_type(option_inner.deref());
            quote! {std::option::Option<#option_inner_name>}
        }
        WrapperType::UnitExpr => quote! {()},
        _ => wrapper.name().parse().unwrap(),
    }
}

fn value_receiver(inner: &WrapperType) -> TokenStream2 {
    match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            format!("value: {inner}").parse().unwrap()
        }
        WrapperType::Bool => "value: bool".parse().unwrap(),
        WrapperType::String => "ptr: *const i8, _len: usize".parse().unwrap(),
        WrapperType::Struct(name) => format!("value: *mut {name}").parse().unwrap(),
        WrapperType::Vec(inner) => format!("value: *mut Vec<{}>", inner.rust_type())
            .parse()
            .unwrap(),
        WrapperType::HashMap(key, value) => format!(
            "value: *mut std::collections::HashMap<{}, {}>",
            key.rust_type(),
            value.rust_type()
        )
        .parse()
        .unwrap(),
        WrapperType::Result(_) => {
            panic!("Vec of Result type not supported yet!")
        }
        WrapperType::Enum(name) => format!("value: {name}").parse().unwrap(),
        WrapperType::UnitExpr => "value: ()".parse().unwrap(),
        WrapperType::Option(inner) => format!("value: *mut Option<{}>", inner.rust_type())
            .parse()
            .unwrap(),
        WrapperType::Trait(_) => {
            panic!("[value_receiver]: Trait objects as vec inner type not supported")
        }
    }
}

fn unwrap_clone_expr(inner: &WrapperType) -> TokenStream2 {
    match &inner {
        WrapperType::String
        | WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Option(_)
        | WrapperType::HashMap(_, _) => quote! {
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
        WrapperType::Trait(_) => panic!("Trait objects in unwrap_clone_expr not supported"),
    }
}

fn received_value_cast(inner: &WrapperType) -> TokenStream2 {
    match inner {
        WrapperType::String => quote! {
            let value = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
        },
        WrapperType::Struct(_) => {
            quote! {
                let value = unsafe { (*value).clone() };
            }
        }
        WrapperType::Vec(_) => {
            quote! {
                let mut new_value = Vec::new();
                std::mem::swap(&mut new_value, unsafe { &mut (*value) });
                let value = new_value;
            }
        }
        WrapperType::Option(_) => {
            quote! {
                let mut new_value = None;
                std::mem::swap(&mut new_value, unsafe { &mut (*value) });
                let value = new_value;
            }
        }
        WrapperType::HashMap(_, _) => {
            quote! {
                let mut new_value = std::collections::HashMap::new();
                std::mem::swap(&mut new_value, unsafe { &mut (*value) });
                let value = new_value;
            }
        }
        _ => quote! {},
    }
}

impl From<&ReusableWrapper> for TokenStream2 {
    fn from(wrapper: &ReusableWrapper) -> TokenStream2 {
        match wrapper {
            ReusableWrapper::Vec(inner) => generate_vec_wrapper(inner),
            ReusableWrapper::Result(inner) => generate_result_wrapper(inner),
            ReusableWrapper::Option(inner) => generate_option_wrapper(inner),
            ReusableWrapper::HashMap(key, value) => generate_hashmap_wrapper(key, value),
        }
    }
}

fn generate_option_wrapper(inner: &WrapperType) -> TokenStream2 {
    let drop_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}drop_{}_option", inner.name());
    let wrapper_fn_name_drop = format_ident!("drop_{}_option", inner.name());

    let unwrap_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}unwrap_{}_option", inner.name());
    let wrapper_fn_name_unwrap = format_ident!("unwrap_{}_option", inner.name());

    let is_some_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}is_some_{}_option", inner.name());
    let wrapper_fn_name_is_some = format_ident!("is_some_{}_option", inner.name());

    let some_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}some_{}_option", inner.name());
    let wrapper_fn_name_some = format_ident!("some_{}_option", inner.name());

    let none_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}none_{}_option", inner.name());
    let wrapper_fn_name_none = format_ident!("none_{}_option", inner.name());

    let inner_type = rust_type(inner);

    let cloned_unwrap_ret_type: TokenStream2 = match &inner {
        WrapperType::String
        | WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Option(_) => {
            quote! {*mut #inner_type}
        }
        WrapperType::HashMap(_, _) => quote! {*mut #inner_type},

        WrapperType::UnitExpr => quote! {()},
        _ => quote! {#inner_type},
    };

    let unwrap_clone_expr = unwrap_clone_expr(inner);

    let some_value_receiver = value_receiver(inner);
    let some_value_cast = received_value_cast(inner);

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #some_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_some(#some_value_receiver) -> *mut std::option::Option<#inner_type> {
            #some_value_cast
            Box::into_raw(Box::new(Some(value)))
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #none_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_none() -> *mut std::option::Option<#inner_type> {
            Box::into_raw(Box::new(None))
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #drop_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_drop(_self: *mut std::option::Option<#inner_type>) {
            unsafe {
                if !_self.is_null() {
                    let _ = Box::from_raw(_self);
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #unwrap_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_unwrap(_self: *mut std::option::Option<#inner_type>) -> #cloned_unwrap_ret_type {
            unsafe {
                match &* _self {
                    Some(value) => {
                        #unwrap_clone_expr
                    },
                    None => panic!("Called unwrap on an None value"),
                }
            }
        }


        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #is_some_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name_is_some(_self: *mut std::option::Option<#inner_type>) -> bool {
            unsafe {
                (* _self).is_some()
            }
        }

    }
}

fn generate_hashmap_wrapper(key: &WrapperType, value: &WrapperType) -> TokenStream2 {
    let map_name = format!("hashmap_{}_{}", key.name(), value.name());
    let map_type: TokenStream2 = format!(
        "std::collections::HashMap<{}, {}>",
        key.rust_type(),
        value.rust_type()
    )
    .parse()
    .expect("valid HashMap type");
    let drop_name = format_ident!("{map_name}_drop");
    let new_name = format_ident!("{map_name}_new");
    let insert_name = format_ident!("{map_name}_insert");
    let len_name = format_ident!("{map_name}_len");
    let key_at_name = format_ident!("{map_name}_key_at");
    let value_at_name = format_ident!("{map_name}_value_at");
    let key_receiver = hashmap_receiver(key, "key");
    let value_receiver = hashmap_receiver(value, "value");
    let key_cast = hashmap_value_cast(key, "key");
    let value_cast = hashmap_value_cast(value, "value");
    let key_return_type = hashmap_get_type(key);
    let value_return_type = hashmap_get_type(value);
    let key_get = hashmap_get_body(key, "key");
    let value_get = hashmap_get_body(value, "value");

    let drop_export = format!("{EXPORTED_SYMBOLS_PREFIX}drop_{map_name}");
    let new_export = format!("{EXPORTED_SYMBOLS_PREFIX}new_{map_name}");
    let insert_export = format!("{EXPORTED_SYMBOLS_PREFIX}insert_{map_name}");
    let len_export = format!("{EXPORTED_SYMBOLS_PREFIX}len_{map_name}");
    let key_at_export = format!("{EXPORTED_SYMBOLS_PREFIX}key_at_{map_name}");
    let value_at_export = format!("{EXPORTED_SYMBOLS_PREFIX}value_at_{map_name}");

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #drop_export)]
        pub unsafe extern "C" fn #drop_name(_self: *mut #map_type) {
            unsafe {
                if !_self.is_null() {
                    let _ = Box::from_raw(_self);
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #new_export)]
        pub extern "C" fn #new_name() -> *mut #map_type {
            Box::into_raw(Box::new(std::collections::HashMap::new()))
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #insert_export)]
        pub unsafe extern "C" fn #insert_name(
            _self: *mut #map_type,
            #key_receiver,
            #value_receiver
        ) {
            #key_cast
            #value_cast
            unsafe {
                (&mut *_self).insert(key, value);
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #len_export)]
        pub unsafe extern "C" fn #len_name(_self: *const #map_type) -> usize {
            unsafe { (&*_self).len() }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #key_at_export)]
        pub unsafe extern "C" fn #key_at_name(_self: *const #map_type, index: usize) -> #key_return_type {
            unsafe {
                let (key, _) = (&*_self).iter().nth(index).expect("HashMap key index out of bounds");
                let key = key.clone();
                #key_get
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #value_at_export)]
        pub unsafe extern "C" fn #value_at_name(_self: *const #map_type, index: usize) -> #value_return_type {
            unsafe {
                let (_, value) = (&*_self).iter().nth(index).expect("HashMap value index out of bounds");
                let value = value.clone();
                #value_get
            }
        }
    }
}

fn hashmap_receiver(wrapper_type: &WrapperType, name: &str) -> TokenStream2 {
    let signature = match wrapper_type {
        WrapperType::IntegerNumber(ty) | WrapperType::FloatingPointNumber(ty) => {
            format!("{name}: {ty}")
        }
        WrapperType::Bool => format!("{name}: bool"),
        WrapperType::String => format!("{name}_ptr: *const i8, _{name}_len: usize"),
        WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Option(_)
        | WrapperType::HashMap(_, _) => {
            format!("{name}: *mut {}", wrapper_type.rust_type())
        }
        WrapperType::Enum(ty) => format!("{name}: {ty}"),
        WrapperType::Result(_) | WrapperType::UnitExpr | WrapperType::Trait(_) => {
            panic!("Unsupported HashMap key or value type: {wrapper_type:?}")
        }
    };
    signature.parse().expect("valid HashMap argument")
}

fn hashmap_value_cast(wrapper_type: &WrapperType, name: &str) -> TokenStream2 {
    match wrapper_type {
        WrapperType::String => {
            let ptr = format_ident!("{name}_ptr");
            let len = format_ident!("_{name}_len");
            let name_ident = format_ident!("{name}");
            quote! {
                let #name_ident = unsafe {
                    let bytes = std::slice::from_raw_parts(#ptr.cast::<u8>(), #len);
                    std::str::from_utf8(bytes)
                        .expect("valid UTF-8 HashMap string")
                        .to_owned()
                };
            }
        }
        WrapperType::Struct(_) => {
            let arg = format_ident!("{name}");
            quote! {
                let #arg = unsafe { (*#arg).clone() };
            }
        }
        WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::HashMap(_, _) => {
            let arg = format_ident!("{name}");
            let new_value = format_ident!("new_{name}");
            quote! {
                let mut #new_value = Default::default();
                std::mem::swap(&mut #new_value, unsafe { &mut (*#arg) });
                let #arg = #new_value;
            }
        }
        WrapperType::Result(_) | WrapperType::UnitExpr | WrapperType::Trait(_) => {
            panic!("Unsupported HashMap key or value type: {wrapper_type:?}")
        }
        _ => quote! {},
    }
}

fn hashmap_get_type(wrapper_type: &WrapperType) -> TokenStream2 {
    match wrapper_type {
        WrapperType::IntegerNumber(ty) | WrapperType::FloatingPointNumber(ty) => {
            ty.parse().expect("valid numeric type")
        }
        WrapperType::Bool => quote! { bool },
        WrapperType::String | WrapperType::Struct(_) => quote! { *mut std::ffi::c_void },
        WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::HashMap(_, _) => {
            let inner_type: TokenStream2 = wrapper_type.rust_type().parse().expect("valid type");
            quote! { *mut #inner_type }
        }
        WrapperType::Enum(ty) => ty.parse().expect("valid enum type"),
        WrapperType::Result(_) | WrapperType::UnitExpr | WrapperType::Trait(_) => {
            panic!("Unsupported HashMap key or value type: {wrapper_type:?}")
        }
    }
}

fn hashmap_get_body(wrapper_type: &WrapperType, ident: &str) -> TokenStream2 {
    let ident = format_ident!("{ident}");
    match wrapper_type {
        WrapperType::String
        | WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Option(_)
        | WrapperType::HashMap(_, _) => quote! {
            Box::into_raw(Box::new(#ident)) as _
        },
        WrapperType::Result(_) | WrapperType::UnitExpr | WrapperType::Trait(_) => {
            panic!("Unsupported HashMap key or value type: {wrapper_type:?}")
        }
        _ => quote! { #ident },
    }
}

fn generate_result_wrapper(inner: &WrapperType) -> TokenStream2 {
    let drop_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}drop_{}_result", inner.name());
    let wrapper_fn_name_drop = format_ident!("drop_{}_result", inner.name());

    let unwrap_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}unwrap_{}_result", inner.name());
    let wrapper_fn_name_unwrap = format_ident!("unwrap_{}_result", inner.name());

    let unwrap_err_ext_fn_name = format!(
        "{EXPORTED_SYMBOLS_PREFIX}unwrap_err_{}_result",
        inner.name()
    );
    let wrapper_fn_name_unwrap_err = format_ident!("unwrap_err_{}_result", inner.name());

    let is_err_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}is_err_{}_result", inner.name());
    let wrapper_fn_name_is_err = format_ident!("is_err_{}_result", inner.name());

    let inner_type: TokenStream2 = rust_type(inner);

    let unwrap_clone_expr = unwrap_clone_expr(inner);

    let cloned_unwrap_ret_type: TokenStream2 = match &inner {
        WrapperType::String
        | WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::HashMap(_, _)
        | WrapperType::Option(_) => {
            quote! {*mut #inner_type}
        }
        WrapperType::UnitExpr => quote! {()},
        _ => quote! {#inner_type},
    };

    let str_err_result_wrapper_name = format!("{}_str_error_result", inner.name());
    let str_err_result_ok_fn_name = format_ident!("{str_err_result_wrapper_name}_ok");
    let str_error_result_ok_fn_ext_fn_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}{str_err_result_ok_fn_name}");

    let value_receiver = value_receiver(inner);
    let value_cast = received_value_cast(inner);

    let str_err_result_err_fn_name = format_ident!("{str_err_result_wrapper_name}_err");
    let str_error_result_err_fn_ext_fn_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}{str_err_result_err_fn_name}");

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #str_error_result_ok_fn_ext_fn_name)]
        pub unsafe extern "C" fn #str_err_result_ok_fn_name(#value_receiver) -> *mut std::result::Result<#inner_type, String> {
            unsafe {
                #value_cast
                let result: std::result::Result<#inner_type, String> = Ok(value);
                Box::into_raw(Box::new(result))
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #str_error_result_err_fn_ext_fn_name)]
        pub unsafe extern "C" fn #str_err_result_err_fn_name(err: *const std::os::raw::c_char) -> *mut std::result::Result<#inner_type, String> {
            unsafe {
                let c_str = std::ffi::CStr::from_ptr(err);
                let str_slice = c_str.to_str().unwrap();
                let result: std::result::Result<#inner_type, String> = Err(str_slice.to_string());
                Box::into_raw(Box::new(result))
            }
        }

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
        pub unsafe extern "C" fn #wrapper_fn_name_unwrap(_self: *mut std::result::Result<#inner_type, std::sync::Arc<dyn std::error::Error>>) -> #cloned_unwrap_ret_type {
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
    let drop_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}drop_{}_vec", inner.name());
    let wrapper_fn_name_drop = format_ident!("drop_{}_vec", inner.name());

    let with_capacity_ext_fn_name = format!(
        "{EXPORTED_SYMBOLS_PREFIX}with_capacity_{}_vec",
        inner.name()
    );
    let wrapper_fn_name_with_capacity = format_ident!("with_capacity_{}_vec", inner.name());

    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}push_{}_vec", inner.name());
    let wrapper_fn_name_push = format_ident!("push_{}_vec", inner.name());

    // New externs for reading returned vectors
    let len_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}len_{}_vec", inner.name());
    let wrapper_fn_name_len = format_ident!("len_{}_vec", inner.name());

    let get_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}get_{}_vec", inner.name());
    let wrapper_fn_name_get = format_ident!("get_{}_vec", inner.name());

    let value_receiver = value_receiver(inner);
    let value_cast = received_value_cast(inner);

    let vec_type: TokenStream2 = format!("Vec<{}>", inner.rust_type()).parse().unwrap();

    // Return type for get() depending on inner type
    let get_return_type: TokenStream2 = match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            inner.parse().unwrap()
        }
        WrapperType::Bool => "bool".parse().unwrap(),
        WrapperType::String => "*mut std::string::String".parse().unwrap(),
        WrapperType::Struct(name) => format!("*mut {name}").parse().unwrap(),
        WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::HashMap(_, _) => {
            let inner_type = rust_type(inner);
            quote! {*mut #inner_type}
        }
        WrapperType::Enum(name) => name.parse().unwrap(),
        WrapperType::Result(_) => {
            panic!("Vec of Result type not supported yet!")
        }
        WrapperType::UnitExpr => panic!("UnitExpr not supported yet as get return type!"),
        WrapperType::Trait(_) => panic!("Trait objects not supported yet as get return type!"),
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
        WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::HashMap(_, _) => quote! {{
            let v = (&*_self)[index].clone();
            Box::into_raw(Box::new(v))
        }},
        WrapperType::Enum(_) => quote! {
            (&*_self)[index]
        },
        WrapperType::Result(_) => {
            panic!("Vec of Result type not supported yet!")
        }
        WrapperType::UnitExpr => {
            panic!("UnitExpr not supported yet as get return type!")
        }
        WrapperType::Trait(_) => {
            panic!("Trait objects not supported yet as get return type!")
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
            ParsedWrapper::Trait(trait_wrapper) => trait_wrapper.name.to_string(),
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
            Wrapper {
                parsed: ParsedWrapper::Trait(trait_wrapper),
                original_definition,
                ..
            } => {
                let tokens: TokenStream2 = trait_wrapper.into();
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
    Trait(TraitWrapper),
}

impl From<Wrapper> for TokenStream {
    fn from(wrapper: Wrapper) -> TokenStream {
        let token_stream: TokenStream2 = (&wrapper).into();
        token_stream.into()
    }
}
