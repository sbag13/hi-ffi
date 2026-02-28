use core::panic;
use std::any::Any;
use std::fmt::Display;
use std::ops::Deref;

use syn::Item;

mod enum_translator;
mod function_translator;
mod impl_translator;
mod struct_translator;
mod trait_translator;

use crate::wrapper::*;
use enum_translator::*;
use function_translator::*;
use impl_translator::*;
use struct_translator::*;
use trait_translator::*;

pub struct NoWrapperErr(pub String);

impl<T: Display> From<T> for NoWrapperErr {
    fn from(value: T) -> Self {
        NoWrapperErr(value.to_string())
    }
}

pub(crate) fn translate(input: Item) -> Result<Wrapper, NoWrapperErr> {
    Ok(match input {
        Item::Struct(item_struct) => translate_struct(item_struct),
        Item::Fn(item_fn) => translate_function(item_fn)?,
        Item::Impl(item_impl) => translate_impl(item_impl)?,
        Item::Enum(item_enum) => translate_enum(item_enum),
        Item::Trait(item_trait) => translate_trait(item_trait)?,
        _ => panic!("Unsupported type: {:?}", input.type_id()),
    })
}

pub(crate) fn map_wrapper_to_reusable(wrapper_type: &WrapperType) -> Vec<ReusableWrapper> {
    match wrapper_type {
        WrapperType::Vec(inner_wrapper_type) => {
            vec![ReusableWrapper::Vec(*inner_wrapper_type.clone())]
        }
        WrapperType::Result(inner_wrapper_type) => {
            let mut wrappers = vec![ReusableWrapper::Result(*inner_wrapper_type.clone())];

            match inner_wrapper_type.deref() {
                WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::Result(_) => {
                    wrappers.extend(map_wrapper_to_reusable(inner_wrapper_type.deref()));
                }
                _ => (),
            }
            wrappers
        }
        WrapperType::Option(inner_wrapper_type) => {
            let mut wrappers = vec![ReusableWrapper::Option(*inner_wrapper_type.clone())];
            match inner_wrapper_type.deref() {
                WrapperType::Vec(_) | WrapperType::Option(_) | WrapperType::Result(_) => {
                    wrappers.extend(map_wrapper_to_reusable(inner_wrapper_type.deref()));
                }
                _ => (),
            }
            wrappers
        }
        _ => vec![],
    }
}
