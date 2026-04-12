use quote::quote;
use std::collections::HashSet;
use std::ops::Deref;
use syn::{FnArg, ItemFn};

use crate::EXPORTED_SYMBOLS_PREFIX;
use crate::translator::{NoWrapperErr, map_wrapper_to_reusable, path_to_wrapper_type};
use crate::wrapper::*;

impl FunctionWrapper {
    pub(crate) fn reusable_wrappers(&self) -> HashSet<ReusableWrapper> {
        let mut reusable_wrappers = self
            .args
            .iter()
            .flat_map(|arg_wrapper| map_wrapper_to_reusable(&arg_wrapper.wrapper_type))
            .collect::<std::collections::HashSet<_>>();

        // Ensure vec reusable wrapper is generated for return vecs too
        if let Some(return_wrapper) = &self.return_wrapper {
            reusable_wrappers.extend(map_wrapper_to_reusable(&return_wrapper.wrapper_type));
        }
        reusable_wrappers
    }
}

pub fn translate_function(item_fn: ItemFn) -> Result<Wrapper, NoWrapperErr> {
    let fn_wrapper = fn_wrapper_from_sig(&item_fn.sig)?;

    Ok(Wrapper {
        original_definition: quote! {#item_fn},
        reusable_wrappers: fn_wrapper.reusable_wrappers(),
        parsed: ParsedWrapper::Function(fn_wrapper),
    })
}

// It ignores the receiver argument
pub(crate) fn fn_wrapper_from_sig(sig: &syn::Signature) -> Result<FunctionWrapper, NoWrapperErr> {
    let fn_name = &sig.ident;
    let args_wrappers = sig
        .inputs
        .iter()
        .filter_map(|arg| map_arg(arg).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    let return_wrapper = return_wrapper(&sig.output)?;

    Ok(FunctionWrapper {
        name: fn_name.clone(),
        extern_function_name: format!("{EXPORTED_SYMBOLS_PREFIX}_{fn_name}"),
        args: args_wrappers,
        return_wrapper,
    })
}

pub fn return_wrapper(
    output: &syn::ReturnType,
) -> Result<Option<FunctionReturnWrapper>, NoWrapperErr> {
    match output {
        syn::ReturnType::Default => Ok(None),
        syn::ReturnType::Type(_, ty) => {
            // Support simple idents and generic types like Vec<T>
            if let syn::Type::Path(path) = ty.deref() {
                let wrapper_type = path_to_wrapper_type(&path.path)?;
                Ok(Some(FunctionReturnWrapper {
                    wrapper_type,
                    return_type: ty.deref().clone(),
                }))
            } else {
                panic!("No path found in return type")
            }
        }
    }
}

pub fn map_arg(arg: &FnArg) -> Result<Option<FunctionArgWrapper>, NoWrapperErr> {
    match arg {
        syn::FnArg::Receiver(_) => Ok(None), // ignore receiver
        syn::FnArg::Typed(pat_type) => {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;
            let arg_name = match pat.deref() {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("Only simple argument names are supported"),
            };
            let wrapper_type = if let syn::Type::Path(path) = ty.deref() {
                path_to_wrapper_type(&path.path)?
            } else {
                panic!("No path found in function argument type")
            };

            Ok(Some(FunctionArgWrapper {
                wrapper_type,
                arg_name,
                arg_type: ty.deref().clone(),
            }))
        }
    }
}
