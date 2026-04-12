use core::panic;
use std::any::Any;
use std::fmt::Display;
use std::ops::Deref;

use quote::ToTokens;
use syn::{GenericArgument, Item, PathSegment, Type};

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
        Item::Struct(item_struct) => translate_struct(item_struct)?,
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

fn path_to_wrapper_type(path: &syn::Path) -> Result<WrapperType, NoWrapperErr> {
    if let Some(ident) = path.get_ident() {
        Ok(ident.to_string().as_str().parse()?)
    } else {
        // Handle non-trivial paths, e.g., Vec<T>
        match path.segments.first() {
            Some(segment) => {
                if let Some(vec_wrapper_type) = segment_as_vec(segment)? {
                    Ok(vec_wrapper_type)
                } else if let Some(result_wrapper_type) = segment_as_result(segment)? {
                    Ok(result_wrapper_type)
                } else if let Some(option_wrapper_type) = segment_as_opt(segment)? {
                    Ok(option_wrapper_type)
                } else if let Some(trait_wrapper_type) = segment_as_boxed_trait(segment)? {
                    Ok(trait_wrapper_type)
                } else {
                    panic!("Unsupported return type: {:?}", segment.ident);
                }
            }
            None => panic!("No segment found in return type"),
        }
    }
}

fn segment_as_boxed_trait(segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
    match segment.ident.to_string().as_str() {
        "Box" => {
            let inner_wrapper_type: WrapperType =
                parse_generic_single_inner_type_from_segment(segment)?;
            match inner_wrapper_type {
                wt @ WrapperType::Trait(_) => Ok(Some(wt)),
                _ => panic!("Returned Box inner type must be a trait"),
            }
        }
        _ => Ok(None),
    }
}

fn segment_as_vec(vec_segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
    match vec_segment.ident.to_string().as_str() {
        "Vec" => {
            let inner_wrapper_type: WrapperType =
                parse_generic_single_inner_type_from_segment(vec_segment)?;

            match inner_wrapper_type {
                WrapperType::Vec(_) => {
                    panic!("Nested vectors are not supported as return types")
                }
                WrapperType::Option(_) => {
                    panic!("Vec of options not supported yet")
                }
                WrapperType::Result(_) => {
                    panic!("Vec of results not supported yet")
                }
                inner => Ok(Some(WrapperType::Vec(Box::new(inner)))),
            }
        }
        _ => Ok(None),
    }
}

fn segment_as_opt(opt_segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
    match opt_segment.ident.to_string().as_str() {
        "Option" => {
            let inner_wrapper_type: WrapperType =
                parse_generic_single_inner_type_from_segment(opt_segment)?;
            match inner_wrapper_type {
                WrapperType::Option(_) => {
                    panic!("Nested options are not supported as return types")
                }
                WrapperType::Result(_) => {
                    panic!("Optional Results not supported yet")
                }
                inner => Ok(Some(WrapperType::Option(Box::new(inner)))),
            }
        }
        _ => Ok(None),
    }
}

fn segment_as_result(result_segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
    match result_segment.ident.to_string().as_str() {
        "Result" => {
            let ok_wrapper_type: WrapperType = match &result_segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    let Some(ok_arg) = args.args.first() else {
                        panic!("No argument found in Result return type");
                    };
                    match ok_arg {
                        GenericArgument::Type(Type::Path(ok_path)) => {
                            if let Some(segment) = ok_path.path.segments.first() {
                                if let Some(vec_wrapper_type) = segment_as_vec(segment)? {
                                    vec_wrapper_type
                                } else {
                                    ok_path.to_token_stream().to_string().as_str().parse()?
                                }
                            } else {
                                panic!("No segment found in Result Ok return type")
                            }
                        }
                        GenericArgument::Type(Type::Tuple(t)) if t.elems.empty_or_trailing() => {
                            WrapperType::UnitExpr
                        }
                        _ => {
                            panic!("Result Ok inner return type must be a path")
                        }
                    }
                }
                _ => panic!("Result return type arguments not supported"),
            };
            Ok(Some(WrapperType::Result(Box::new(ok_wrapper_type))))
        }
        _ => Ok(None),
    }
}

fn parse_generic_single_inner_type_from_segment(
    segment: &PathSegment,
) -> Result<WrapperType, NoWrapperErr> {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            let Some(inner_arg) = args.args.first() else {
                panic!("No argument found in generic's arguments");
            };
            match inner_arg {
                GenericArgument::Type(Type::Path(inner_path)) => {
                    Ok(inner_path.to_token_stream().to_string().as_str().parse()?)
                }
                GenericArgument::Type(Type::TraitObject(type_trait_obj)) => {
                    if type_trait_obj.bounds.len() != 1 {
                        panic!(
                            "Only one trait bound is supported in trait objects for function arguments"
                        );
                    }
                    let first_bound = type_trait_obj.bounds.first().unwrap();

                    match first_bound {
                        syn::TypeParamBound::Trait(trait_bound) => {
                            let trait_ident = &trait_bound.path.segments.last().unwrap().ident;
                            Ok(WrapperType::Trait(trait_ident.to_string()))
                        }
                        _ => panic!(
                            "Only trait bounds are supported in trait objects for function arguments"
                        ),
                    }
                }
                _ => panic!("Generic's inner arg type must be a path"),
            }
        }
        _ => panic!("Generic arguments not supported"),
    }
}
