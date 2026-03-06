use std::collections::HashSet;
use std::fmt::Display;
use std::ops::Deref;

use quote::ToTokens;
use syn::ItemImpl;

use crate::translator::{NoWrapperErr, map_arg, map_wrapper_to_reusable};
use crate::wrapper::Wrapper;
use crate::wrapper::impl_block_wrapper::MethodWrapper;
use crate::{EXPORTED_SYMBOLS_PREFIX, ReusableWrapper};

use super::impl_block_wrapper::ImplBlockWrapper;
use super::{ParsedWrapper, return_wrapper};

pub fn translate_impl(item_impl: ItemImpl) -> Result<Wrapper, NoWrapperErr> {
    let struct_name = if let syn::Type::Path(path) = item_impl.self_ty.deref() {
        if let Some(ident) = path.path.get_ident() {
            ident.to_owned()
        } else {
            panic!("No ident found")
        }
    } else {
        panic!("Self type is not a path")
    };

    let mut reusable_wrappers = HashSet::new();

    let methods: Vec<_> = item_impl
        .items
        .iter()
        .map(|item| {
            if let syn::ImplItem::Fn(method) = item {
                trait_method_wrapper_from_signature(
                    &method.sig,
                    matches!(method.vis, syn::Visibility::Public(_)),
                    &mut reusable_wrappers,
                    &struct_name,
                )
            } else {
                panic!("Unsupported impl item")
            }
        })
        .collect::<Result<Vec<MethodWrapper>, NoWrapperErr>>()?;

    reusable_wrappers.extend(
        methods
            .iter()
            .filter_map(|method| {
                // Ensure vec reusable wrapper is generated for return vecs too
                method
                    .return_wrapper
                    .as_ref()
                    .map(|fn_wrapper| map_wrapper_to_reusable(&fn_wrapper.wrapper_type))
            })
            .flatten(),
    );

    Ok(Wrapper {
        original_definition: item_impl.into_token_stream(),
        parsed: ParsedWrapper::ImplBlock(ImplBlockWrapper {
            struct_name,
            methods,
        }),
        reusable_wrappers,
    })
}

pub(crate) fn trait_method_wrapper_from_signature(
    sig: &syn::Signature,
    public: bool,
    reusable_wrappers: &mut HashSet<ReusableWrapper>,
    struct_name: impl Display,
) -> Result<MethodWrapper, NoWrapperErr> {
    let args = sig
        .inputs
        .iter()
        .filter_map(|arg| map_arg(arg).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    for arg in &args {
        reusable_wrappers.extend(map_wrapper_to_reusable(&arg.wrapper_type));
    }

    let return_wrapper: Option<crate::wrapper::FunctionReturnWrapper> =
        return_wrapper(&sig.output)?;

    if let Some(return_wrapper) = &return_wrapper {
        reusable_wrappers.extend(map_wrapper_to_reusable(&return_wrapper.wrapper_type));
    }

    Ok(MethodWrapper {
        name: sig.ident.clone(),
        extern_function_name: format!("{EXPORTED_SYMBOLS_PREFIX}_{struct_name}_{}", sig.ident),
        public,
        is_static: sig.receiver().is_none(),
        args,
        return_wrapper,
    })
}
