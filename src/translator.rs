use core::panic;
use std::any::Any;

use syn::Item;

mod function_translator;
mod struct_translator;

use crate::wrapper::*;
use function_translator::*;
use struct_translator::*;

pub(crate) fn translate(input: Item) -> Wrapper {
    match input {
        Item::Struct(item_struct) => translate_struct(item_struct),
        Item::Fn(item_fn) => translate_function(item_fn),
        _ => panic!("Unsupported type: {:?}", input.type_id()),
    }
}
