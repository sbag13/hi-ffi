use std::fmt::Debug;

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::wrapper::{
    FunctionWrapper, MappedFunctionArgsTokens, MappedReturnType, WrapperType,
    map_function_arg_wrappers, map_return_type,
};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::ItemTrait;

pub struct TraitWrapper {
    pub name: Ident,
    pub functions: Vec<FunctionWrapper>,
    pub original_item_trait: ItemTrait,
}

impl Debug for TraitWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TraitWrapper {{ name: {}, functions: {:?} }}",
            self.name, self.functions
        )
    }
}

impl From<&TraitWrapper> for TokenStream2 {
    fn from(trait_wrapper: &TraitWrapper) -> TokenStream2 {
        let TraitWrapper {
            name, functions, ..
        } = &trait_wrapper;

        let bridge_name = format_ident!("{}Bridge", name);
        let vtable_name = format_ident!("{}VTable", name);

        let vtable_functions = functions.iter().map(|function| {
            let FunctionWrapper { name, .. } = function;

            let args = function.args.iter().map(|arg| {
                let arg_name = &arg.arg_name;
                let arg_type = match arg.wrapper_type {
                    WrapperType::Bool
                    | WrapperType::IntegerNumber(_)
                    | WrapperType::FloatingPointNumber(_)
                    | WrapperType::Enum(_) => {
                        let arg_type = &arg.arg_type;
                        quote! { #arg_type }
                    }
                    _ => quote! { *mut std::ffi::c_void },
                };
                quote! { #arg_name: #arg_type }
            });

            let return_type_sig = match &function.return_wrapper {
                Some(ret) => match ret.wrapper_type {
                    WrapperType::Bool
                    | WrapperType::IntegerNumber(_)
                    | WrapperType::FloatingPointNumber(_)
                    | WrapperType::Enum(_) => {
                        let return_type = &ret.return_type;
                        quote! { -> #return_type }
                    }
                    WrapperType::Struct(_)
                    | WrapperType::String
                    | WrapperType::Vec(_)
                    | WrapperType::Option(_)
                    | WrapperType::Result(_) => {
                        quote! { -> *mut std::ffi::c_void }
                    }

                    _ => quote! {},
                },
                None => quote! {},
            };

            quote! {
                pub #name: extern "C" fn(*mut std::ffi::c_void, #(#args),*) #return_type_sig,
            }
        });

        let impl_functions = functions.iter().map(|function| {
            let FunctionWrapper { name, return_wrapper, .. } = function;

            let args = function.args.iter().map(|arg| {
                let arg_name = &arg.arg_name;
                let arg_type = &arg.arg_type;
                quote! { #arg_name: #arg_type }
            });

            let arg_names = function.args.iter().map(|arg| {
                let arg_name = &arg.arg_name;
                quote! { #arg_name }
            });

            let arg_casts = function.args.iter().map(|arg| {
                let arg_name = &arg.arg_name;
                match arg.wrapper_type {
                    WrapperType::String | WrapperType::Struct(_) | WrapperType::Vec(_) | WrapperType::Option(_) => {
                        quote! {
                            let #arg_name = Box::into_raw(Box::new(#arg_name)) as *mut std::ffi::c_void;
                        }
                    },
                    _ => quote! {},
                }
            });

            match return_wrapper {
                Some(ret) => {
                    let original_return_type = &ret.return_type;
                    let return_conversion = match ret.wrapper_type {
                        WrapperType::Bool
                        | WrapperType::IntegerNumber(_)
                        | WrapperType::FloatingPointNumber(_)
                        | WrapperType::Enum(_)  => {
                            quote! { result }
                        }
                        WrapperType::String |
                        WrapperType::Struct(_) |
                        WrapperType::Vec(_) |
                        WrapperType::Option(_) |
                        WrapperType::Result(_) => {
                              quote! { *Box::from_raw(result as *mut #original_return_type) }
                        }
                        _ => quote! { result },
                    };

                    quote! {
                        fn #name(&self, #(#args),*) -> #original_return_type {
                            unsafe {
                                let func = (*self.vtable).#name;
                                #(#arg_casts)*
                                let result = func(self.obj, #(#arg_names),*);
                                #return_conversion
                            }
                        }
                    }
                }
                None => {
                    quote! {
                        fn #name(&self, #(#args),*) {
                            unsafe {
                                let func = (*self.vtable).#name;
                                #(#arg_casts)*
                                func(self.obj, #(#arg_names),*);
                            }
                        }
                    }
                }
            }
        });

        let box_dyn_functions = functions.iter().map(|function| {
            let ext_name = format!("{}_BoxDyn", function.extern_function_name);
            let fn_name = &function.name;
            let bridge_fn_name = format_ident!("{}_bridge", &function.name);

            let MappedFunctionArgsTokens {
                arg_signatures,
                arg_names,
                arg_casts,
            } = map_function_arg_wrappers(function.args.iter());

            let MappedReturnType {
                return_type_sig,
                result_cast,
            } = map_return_type(&function.return_wrapper);

            quote! {
                #[doc(hidden)]
                #[unsafe(no_mangle)]
                #[unsafe(export_name = #ext_name)]
                pub unsafe extern "C" fn #bridge_fn_name(obj: *mut std::ffi::c_void, #(#arg_signatures,)*) #return_type_sig {
                    #(#arg_casts)*
                    let result = (*(obj as *mut Box<dyn #name>)).#fn_name(#(#arg_names,)*);
                    #result_cast
                }
            }
        });

        let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}{name}_BoxDyn_drop");
        let drop_wrapper_ident = format_ident!("{name}_BoxDyn_drop");
        let box_dyn_drop = quote! {
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            #[unsafe(export_name = #drop_ext_name)]
            pub unsafe extern "C" fn #drop_wrapper_ident(obj: *mut std::ffi::c_void) {
                drop(Box::from_raw(obj as *mut Box<dyn #name>));
            }
        };

        quote! {
            #[repr(C)]
            pub struct #vtable_name {
                #(#vtable_functions)*
            }

            #[repr(C)]
            #[derive(Debug)]
            pub struct #bridge_name {
                pub obj: *mut std::ffi::c_void,
                pub vtable: *const #vtable_name,
                pub deleter: extern "C" fn(*mut std::ffi::c_void),
            }

            impl #name for #bridge_name {
                #(#impl_functions)*
            }

            impl Drop for #bridge_name {
                fn drop(&mut self) {
                    (self.deleter)(self.obj);
                }
            }

            #(#box_dyn_functions)*

            #box_dyn_drop
        }
    }
}
