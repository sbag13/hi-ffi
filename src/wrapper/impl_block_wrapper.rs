use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use crate::wrapper::{
    DefaultConstructor, MappedFunctionArgsTokens, MappedReturnType, gen_default_constructor,
    map_function_arg_wrappers, map_return_type,
};

use super::{FunctionArgWrapper, FunctionReturnWrapper};

#[derive(Debug)]
pub struct ImplBlockWrapper {
    pub(crate) struct_name: Ident,
    pub(crate) methods: Vec<MethodWrapper>,
    pub(crate) default_constructor: Option<DefaultConstructor>,
}

#[derive(Debug)]
pub struct MethodWrapper {
    pub(crate) name: Ident,
    pub(crate) extern_function_name: String,
    pub(crate) public: bool,
    pub(crate) is_static: bool,
    pub(crate) args: Vec<FunctionArgWrapper>,
    pub(crate) return_wrapper: Option<FunctionReturnWrapper>,
}

impl From<&ImplBlockWrapper> for TokenStream2 {
    fn from(impl_block_wrapper: &ImplBlockWrapper) -> Self {
        let struct_name = &impl_block_wrapper.struct_name;

        let methods = impl_block_wrapper
            .methods
            .iter()
            .filter(|&method_wrapper| method_wrapper.public)
            .map(|method_wrapper| {
                let method_name = &method_wrapper.name;
                let extern_function_name = &method_wrapper.extern_function_name;
                let wrapper_name = format_ident!(
                    "ffi_wrapper_{}_{}",
                    struct_name.to_string().to_lowercase(),
                    method_name
                );
                let MappedReturnType {
                    return_type_sig,
                    result_cast,
                } = map_return_type(&method_wrapper.return_wrapper);
                let MappedFunctionArgsTokens {
                    arg_signatures,
                    arg_names,
                    arg_casts,
                } = map_function_arg_wrappers(method_wrapper.args.iter());
                let receiver = if method_wrapper.is_static {
                    quote! {}
                } else {
                    quote! {_self: *mut #struct_name,}
                };
                let target_object = if method_wrapper.is_static {
                    quote! {#struct_name::}
                } else {
                    quote! {(&mut *_self).}
                };
                quote! {
                    #[doc(hidden)]
                    #[unsafe(export_name = #extern_function_name)]
                    pub unsafe extern "C" fn #wrapper_name( #receiver #(#arg_signatures,)*) #return_type_sig {
                        #(#arg_casts)*
                        let result = #target_object #method_name(#(#arg_names,)*);
                        #result_cast
                    }
                }
            })
            .collect::<Vec<_>>();

        let default_constructor =
            gen_default_constructor(&impl_block_wrapper.default_constructor, struct_name);

        quote! {
            #(#methods)*
            #default_constructor
        }
    }
}
