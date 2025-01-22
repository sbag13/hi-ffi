use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

#[derive(Debug)]
pub struct ImplBlockWrapper {
    pub(crate) struct_name: Ident,
    pub(crate) methods: Vec<MethodWrapper>,
}

#[derive(Debug)]
pub struct MethodWrapper {
    pub(crate) name: Ident,
    pub(crate) extern_function_name: String,
    pub(crate) public: bool,
    // pub(crate) args: Vec<MethodArgWrapper>,
    // pub(crate) return_type: Type,
}

impl From<&ImplBlockWrapper> for TokenStream2 {
    fn from(impl_block_wrapper: &ImplBlockWrapper) -> Self {
        let struct_name = &impl_block_wrapper.struct_name;
        let methods = impl_block_wrapper
            .methods
            .iter()
            .filter_map(|method| {
                method.public.then(|| {

                let method_name = &method.name;
                let extern_function_name = &method.extern_function_name;
                let wrapper_name = format_ident!("ffi_wrapper_{}_{}", struct_name, method_name);
                let arg_signatures: Vec<TokenStream2> = vec![]; // TODO
                let return_type_sig = quote! {-> ()}; // TODO
                let arg_casts: Vec<TokenStream2> = vec![]; // TODO
                let arg_names: Vec<TokenStream2> = vec![]; // TODO
                let result_cast = quote! {result}; // TODO
                quote! {
                    #[doc(hidden)]
                    #[export_name = #extern_function_name]
                    pub unsafe extern "C" fn #wrapper_name(_self: *mut #struct_name, #(#arg_signatures,)*) #return_type_sig {
                        #(#arg_casts)*
                        let result = (&mut *_self).#method_name(#(#arg_names,)*);
                        #result_cast
                    }
                }})
            })
            .collect::<Vec<_>>();

        quote! {


            #(#methods)*
        }
    }
}
