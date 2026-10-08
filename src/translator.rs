use core::panic;
use std::any::Any;
use std::fmt::Display;

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
            let mut wrappers = vec![ReusableWrapper::Vec(*inner_wrapper_type.clone())];
            wrappers.extend(map_wrapper_to_reusable(inner_wrapper_type));
            wrappers
        }
        WrapperType::Result(inner_wrapper_type) => {
            let mut wrappers = vec![ReusableWrapper::Result(*inner_wrapper_type.clone())];
            wrappers.extend(map_wrapper_to_reusable(inner_wrapper_type));
            wrappers
        }
        WrapperType::Option(inner_wrapper_type) => {
            let mut wrappers = vec![ReusableWrapper::Option(*inner_wrapper_type.clone())];
            wrappers.extend(map_wrapper_to_reusable(inner_wrapper_type));
            wrappers
        }
        _ => vec![],
    }
}

fn path_to_wrapper_type(path: &syn::Path) -> Result<WrapperType, NoWrapperErr> {
    path_to_wrapper_type_with_result(path, true)
}

pub(crate) fn path_to_wrapper_type_without_result(
    path: &syn::Path,
) -> Result<WrapperType, NoWrapperErr> {
    path_to_wrapper_type_with_result(path, false)
}

fn path_to_wrapper_type_with_result(
    path: &syn::Path,
    allow_result: bool,
) -> Result<WrapperType, NoWrapperErr> {
    if let Some(ident) = path.get_ident() {
        Ok(ident.to_string().as_str().parse()?)
    } else {
        match path.segments.last() {
            Some(segment) => {
                if let Some(vec_wrapper_type) = segment_as_vec(segment)? {
                    Ok(vec_wrapper_type)
                } else if let Some(option_wrapper_type) = segment_as_opt(segment)? {
                    Ok(option_wrapper_type)
                } else if allow_result
                    && let Some(result_wrapper_type) = segment_as_result(segment)?
                {
                    Ok(result_wrapper_type)
                } else if let Some(trait_wrapper_type) = segment_as_boxed_trait(segment)? {
                    Ok(trait_wrapper_type)
                } else {
                    Ok(segment.ident.to_string().as_str().parse()?)
                }
            }
            None => panic!("No segment found in return type"),
        }
    }
}

fn parse_inner_wrapper_type(segment: &PathSegment) -> Result<WrapperType, NoWrapperErr> {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            let Some(GenericArgument::Type(Type::Path(inner_path))) = args.args.first() else {
                panic!("Generic's inner arg type must be a path");
            };
            path_to_wrapper_type_with_result(&inner_path.path, false)
        }
        _ => panic!("Generic arguments not supported"),
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
            let inner_wrapper_type = parse_inner_wrapper_type(vec_segment)?;
            Ok(Some(WrapperType::Vec(Box::new(inner_wrapper_type))))
        }
        _ => Ok(None),
    }
}

fn segment_as_opt(opt_segment: &PathSegment) -> Result<Option<WrapperType>, NoWrapperErr> {
    match opt_segment.ident.to_string().as_str() {
        "Option" => {
            let inner_wrapper_type = parse_inner_wrapper_type(opt_segment)?;
            Ok(Some(WrapperType::Option(Box::new(inner_wrapper_type))))
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
                            path_to_wrapper_type_with_result(&ok_path.path, false)?
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

#[cfg(test)]
mod tests {
    use super::{WrapperType, path_to_wrapper_type, path_to_wrapper_type_without_result};

    #[test]
    fn parses_result_only_at_the_outer_return_position() {
        let path = syn::parse_str::<syn::Path>("Result<Option<Vec<i32>>, Error>").unwrap();
        let wrapper = path_to_wrapper_type(&path).unwrap_or_else(|_| panic!("type was rejected"));
        assert!(matches!(
            &wrapper,
            WrapperType::Result(inner)
                if matches!(inner.as_ref(), WrapperType::Option(option)
                    if matches!(option.as_ref(), WrapperType::Vec(vec)
                        if matches!(vec.as_ref(), WrapperType::IntegerNumber(_))))
        ));
    }

    #[test]
    fn rejects_result_arguments_and_nested_results() {
        let result_path = syn::parse_str::<syn::Path>("Result<Vec<i32>, Error>").unwrap();
        assert!(path_to_wrapper_type_without_result(&result_path).is_err());

        let nested_path = syn::parse_str::<syn::Path>("Vec<Option<Result<i32, Error>>>").unwrap();
        assert!(path_to_wrapper_type(&nested_path).is_err());
    }
}
