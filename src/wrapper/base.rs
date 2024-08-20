use crate::TokenStream2;
use quote::quote;

pub const SLICE_GET_LEN_FN_NAME: &str = "__hiFfi__$slice_len";
pub const SLICE_GET_PTR_FN_NAME: &str = "__hiFfi__$slice_ptr";
pub const SLICE_DROP_FN_NAME: &str = "__hiFfi__$slice_drop";

pub const RUST_STRING_DROP_FN_NAME: &str = "__hiFfi__$rust_string_drop";
pub const RUST_STRING_DATA_FN_NAME: &str = "__hiFfi__$rust_string_data";
pub const RUST_STRING_LEN_FN_NAME: &str = "__hiFfi__$rust_string_len";

pub fn rust_code_base() -> TokenStream2 {
    quote! {
        #[repr(C)]
        pub struct FfiSlice {
            pub ptr: *const u8,
            pub len: usize,
        }

        #[doc(hidden)]
        #[export_name = #SLICE_GET_LEN_FN_NAME]
        pub unsafe extern "C" fn slice_len(_self: *const FfiSlice) -> usize {
            unsafe {
                (*_self).len
            }
        }

        #[doc(hidden)]
        #[export_name = #SLICE_GET_PTR_FN_NAME]
        pub unsafe extern "C" fn slice_ptr(_self: *const FfiSlice) -> *const std::os::raw::c_char {
            unsafe {
                (*_self).ptr as *const std::os::raw::c_char
            }
        }

        #[doc(hidden)]
        #[export_name = #SLICE_DROP_FN_NAME]
        pub unsafe extern "C" fn slice_drop(_self: *mut FfiSlice) {
            unsafe {
                let _ = Box::from_raw(_self);
            }
        }

        #[doc(hidden)]
        #[export_name = #RUST_STRING_DROP_FN_NAME]
        pub unsafe extern "C" fn rust_string_drop(_self: *mut std::ffi::c_void) {
            unsafe {
                let _ = Box::from_raw(_self as *mut String);
            }
        }

        #[doc(hidden)]
        #[export_name = #RUST_STRING_DATA_FN_NAME]
        pub unsafe extern "C" fn rust_string_data(_self: *const std::ffi::c_void) -> *const u8 {
            unsafe {
                let s = &*( _self as *const String);
                s.as_ptr()
            }
        }

        #[doc(hidden)]
        #[export_name = #RUST_STRING_LEN_FN_NAME]
        pub unsafe extern "C" fn rust_string_len(_self: *const std::ffi::c_void) -> usize {
            unsafe {
                let s = &*( _self as *const String);
                s.len()
            }
        }
    }
}
