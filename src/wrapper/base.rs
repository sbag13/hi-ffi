use crate::TokenStream2;
use quote::quote;

pub const SLICE_GET_LEN_FN_NAME: &str = "hiFfi__slice_len";
pub const SLICE_GET_PTR_FN_NAME: &str = "hiFfi__slice_ptr";
pub const SLICE_DROP_FN_NAME: &str = "hiFfi__slice_drop";

pub const RUST_STRING_DROP_FN_NAME: &str = "hiFfi__rust_string_drop";
pub const RUST_STRING_DATA_FN_NAME: &str = "hiFfi__rust_string_data";
pub const RUST_STRING_LEN_FN_NAME: &str = "hiFfi__rust_string_len";
pub const RUST_STRING_FROM_C_PTR_FN_NAME: &str = "hiFfi__rust_string_from_c_ptr";

pub const RUST_ARC_DYN_ERR_DROP_FN_NAME: &str = "hiFfi__rust_arc_dyn_err_drop";
pub const RUST_ARC_DYN_ERR_DESC_FN_NAME: &str = "hiFfi__rust_arc_dyn_err_desc";
pub const RUST_ARC_DYN_ERR_SOURCE_FN_NAME: &str = "hiFfi__rust_arc_dyn_err_source";
pub const RUST_ARC_DYN_ERR_CLONE_FN_NAME: &str = "hiFfi__rust_arc_dyn_err_clone";

pub const RUST_REF_DYN_ERR_DROP_FN_NAME: &str = "hiFfi__rust_ref_dyn_err_drop";
pub const RUST_REF_DYN_ERR_DESC_FN_NAME: &str = "hiFfi__rust_ref_dyn_err_desc";
pub const RUST_REF_DYN_ERR_SOURCE_FN_NAME: &str = "hiFfi__rust_ref_dyn_err_source";

pub fn rust_code_base() -> TokenStream2 {
    quote! {
        #[doc(hidden)]
        #[unsafe(export_name = #RUST_STRING_FROM_C_PTR_FN_NAME)]
        pub unsafe extern "C" fn rust_string_from_c_ptr(ptr: *const std::os::raw::c_char) -> *mut std::ffi::c_void {
            unsafe {
                let c_str = std::ffi::CStr::from_ptr(ptr);
                let string = c_str.to_str().unwrap().to_owned();
                Box::into_raw(Box::new(string)) as *mut std::ffi::c_void
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_REF_DYN_ERR_DROP_FN_NAME)]
        pub unsafe extern "C" fn rust_ref_dyn_err_drop(_self: *const &dyn std::error::Error) {
            unsafe {
                let _ = Box::from_raw(_self as *mut &dyn std::error::Error);
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_REF_DYN_ERR_DESC_FN_NAME)]
        pub unsafe extern "C" fn rust_ref_dyn_err_desc(_self: *const &dyn std::error::Error) -> *const String {
            unsafe {
                let err = &*(_self);
                Box::into_raw(Box::new(err.to_string())) as _
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_REF_DYN_ERR_SOURCE_FN_NAME)]
        pub unsafe extern "C" fn rust_ref_dyn_err_source(_self: *const &dyn std::error::Error) -> *const std::ffi::c_void {
            unsafe {
                let err = &*(_self);
                match err.source() {
                    Some(source_err) => {
                        Box::into_raw(Box::new(source_err.clone())) as *const std::ffi::c_void
                    },
                    None => std::ptr::null(),
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_ARC_DYN_ERR_CLONE_FN_NAME)]
        pub unsafe extern "C" fn rust_arc_dyn_err_clone(_self: *const std::sync::Arc<dyn std::error::Error>) -> *mut std::sync::Arc<dyn std::error::Error> {
            unsafe {
                let arc_err = &*(_self);
                let arc_err_clone = std::sync::Arc::clone(arc_err);
                Box::into_raw(Box::new(arc_err_clone))
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_ARC_DYN_ERR_DROP_FN_NAME)]
        pub unsafe extern "C" fn rust_arc_dyn_err_drop(_self: *mut std::sync::Arc<dyn std::error::Error>) {
            unsafe {
                let _ = Box::from_raw(_self);
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_ARC_DYN_ERR_DESC_FN_NAME)]
        pub unsafe extern "C" fn rust_arc_dyn_err_desc(_self: *const std::sync::Arc<dyn std::error::Error>) -> *const String {
            unsafe {
                let err = &*(_self);
                Box::into_raw(Box::new(err.to_string())) as _
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_ARC_DYN_ERR_SOURCE_FN_NAME)]
        pub unsafe extern "C" fn rust_arc_dyn_err_source(_self: *const std::sync::Arc<dyn std::error::Error>) -> *const std::ffi::c_void {
            unsafe {
                let err = &*(_self);
                match err.source() {
                    Some(source_err) => {
                        Box::into_raw(Box::new(source_err.clone())) as *const std::ffi::c_void
                    },
                    None => std::ptr::null(),
                }
            }
        }

        #[repr(C)]
        pub struct FfiSlice {
            pub ptr: *const u8,
            pub len: usize,
        }

        #[doc(hidden)]
        #[unsafe(export_name = #SLICE_GET_LEN_FN_NAME)]
        pub unsafe extern "C" fn slice_len(_self: *const FfiSlice) -> usize {
            unsafe {
                (*_self).len
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #SLICE_GET_PTR_FN_NAME)]
        pub unsafe extern "C" fn slice_ptr(_self: *const FfiSlice) -> *const std::os::raw::c_char {
            unsafe {
                (*_self).ptr as *const std::os::raw::c_char
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #SLICE_DROP_FN_NAME)]
        pub unsafe extern "C" fn slice_drop(_self: *mut FfiSlice) {
            unsafe {
                let _ = Box::from_raw(_self);
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_STRING_DROP_FN_NAME)]
        pub unsafe extern "C" fn rust_string_drop(_self: *mut std::ffi::c_void) {
            unsafe {
                let _ = Box::from_raw(_self as *mut String);
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_STRING_DATA_FN_NAME)]
        pub unsafe extern "C" fn rust_string_data(_self: *const std::ffi::c_void) -> *const u8 {
            unsafe {
                let s = &*( _self as *const String);
                s.as_ptr()
            }
        }

        #[doc(hidden)]
        #[unsafe(export_name = #RUST_STRING_LEN_FN_NAME)]
        pub unsafe extern "C" fn rust_string_len(_self: *const std::ffi::c_void) -> usize {
            unsafe {
                let s = &*( _self as *const String);
                s.len()
            }
        }
    }
}
