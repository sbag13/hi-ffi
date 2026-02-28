use quote::quote;
use std::collections::HashSet;
use syn::ItemTrait;

use crate::Wrapper;
use crate::translator::NoWrapperErr;
use crate::translator::function_translator::fn_wrapper_from_sig;
use crate::wrapper::ParsedWrapper;
use crate::wrapper::trait_wrapper::TraitWrapper;

pub fn translate_trait(item_trait: ItemTrait) -> Result<Wrapper, NoWrapperErr> {
    if !item_trait.attrs.is_empty() {
        panic!("Attributes on traits are not supported");
    }
    if !item_trait.generics.params.is_empty() {
        panic!("Generic traits are not supported");
    }

    let mut reusable_wrappers = HashSet::new();

    let functions = item_trait
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(fn_item) = item {
                let fn_wrapper = match fn_wrapper_from_sig(&fn_item.sig) {
                    Err(err) => return Some(Err(err)),
                    Ok(wrapper) => wrapper,
                };

                reusable_wrappers.extend(fn_wrapper.reusable_wrappers());

                Some(Ok(fn_wrapper))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Wrapper {
        original_definition: quote! {#item_trait},
        parsed: ParsedWrapper::Trait(TraitWrapper {
            name: item_trait.ident.clone(),
            functions,
            original_item_trait: item_trait,
        }),
        reusable_wrappers,
    })
}
