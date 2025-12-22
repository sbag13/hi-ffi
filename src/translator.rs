use core::panic;
use std::any::Any;

use syn::Item;

mod enum_translator;
mod function_translator;
mod impl_translator;
mod struct_translator;

use crate::wrapper::*;
use enum_translator::*;
use function_translator::*;
use impl_translator::*;
use struct_translator::*;

pub(crate) fn translate(input: Item) -> Wrapper {
    match input {
        Item::Struct(item_struct) => translate_struct(item_struct),
        Item::Fn(item_fn) => translate_function(item_fn),
        Item::Impl(item_impl) => translate_impl(item_impl),
        Item::Enum(item_enum) => translate_enum(item_enum),
        _ => panic!("Unsupported type: {:?}", input.type_id()),
    }
}
