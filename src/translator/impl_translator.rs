use std::ops::Deref;

use quote::ToTokens;
use syn::ItemImpl;

use crate::{
    translator::map_arg,
    wrapper::{impl_block_wrapper::MethodWrapper, Wrapper},
    EXPORTED_SYMBOLS_PREFIX,
};

use super::{impl_block_wrapper::ImplBlockWrapper, return_wrapper, ParsedWrapper};

pub fn translate_impl(item_impl: ItemImpl) -> Wrapper {
    let struct_name = if let syn::Type::Path(path) = item_impl.self_ty.deref() {
        if let Some(ident) = path.path.get_ident() {
            ident.to_owned()
        } else {
            panic!("No ident found")
        }
    } else {
        panic!("Self type is not a path")
    };

    let methods = item_impl
        .items
        .iter()
        .map(|item| {
            if let syn::ImplItem::Fn(method) = item {
                MethodWrapper {
                    name: method.sig.ident.clone(),
                    extern_function_name: format!(
                        "{EXPORTED_SYMBOLS_PREFIX}_{}_{}",
                        struct_name, method.sig.ident
                    ),
                    public: matches!(method.vis, syn::Visibility::Public(_)),
                    is_static: method.sig.receiver().is_none(),
                    args: method
                        .sig
                        .inputs
                        .iter()
                        .filter(|arg| !matches!(arg, syn::FnArg::Receiver(_))) // ignore receiver
                        .map(map_arg)
                        .collect::<Vec<_>>(),
                    return_wrapper: return_wrapper(&method.sig.output),
                }
            } else {
                panic!("Unsupported impl item")
            }
        })
        .collect();

    Wrapper {
        original_definition: item_impl.into_token_stream(),
        parsed: ParsedWrapper::ImplBlock(ImplBlockWrapper {
            struct_name,
            methods,
        }),
    }
}
