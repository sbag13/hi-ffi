use std::fmt::{Debug, Display};

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::Type;

use crate::wrapper::WrapperType;

pub struct StructWrapper {
    pub(crate) name: Ident,
    pub(crate) fields: Vec<FieldWrapper>,
    pub(crate) default_constructor: Option<DefaultConstructor>, // name of the default constructor if it exists
    pub(crate) drop_ext_fn_name: String,
    pub(crate) clone_ext_fn_name: String,
    pub(crate) original_item_struct: syn::ItemStruct,
}

impl Debug for StructWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructWrapper")
            .field("name", &self.name)
            .finish()
    }
}

impl From<&StructWrapper> for TokenStream2 {
    fn from(struct_wrapper: &StructWrapper) -> TokenStream2 {
        let class_name = &struct_wrapper.name;

        let fields = struct_wrapper
            .fields
            .iter()
            .map(|field| match &field.wrapper_type {
                WrapperType::IntegerNumber(_)
                | WrapperType::FloatingPointNumber(_)
                | WrapperType::Enum(_)
                | WrapperType::Bool => map_primitive_field(field, class_name),
                WrapperType::String => map_string_field(field, class_name),
                WrapperType::Struct(_) => map_custom_field(field, class_name),
                WrapperType::Vec(_) => map_vec_field(field, class_name),
                WrapperType::Option(_) => map_option_field(field, class_name),
                WrapperType::Result(wrapper_type) => {
                    todo!(
                        "Result wrapper type is not supported yet as struct field: {wrapper_type:?}"
                    )
                }
                WrapperType::UnitExpr => {
                    panic!("UnitExpr wrapper type is not supported for struct fields")
                }
                WrapperType::Trait(_) => {
                    panic!("Trait wrapper type is not supported for struct fields")
                }
            });

        let default_constructor =
            gen_default_constructor(&struct_wrapper.default_constructor, class_name);
        let drop = gen_drop(&struct_wrapper.drop_ext_fn_name, class_name);
        let clone = gen_clone(&struct_wrapper.clone_ext_fn_name, class_name);

        let mut item_struct = struct_wrapper.original_item_struct.clone();
        item_struct.fields.iter_mut().for_each(|field| {
            field.attrs.retain(|attr| !attr.path().is_ident("ffi"));
        });
        let struct_definition = quote! {#item_struct};

        quote! {
            #struct_definition
            #(#fields)*
            #default_constructor
            #drop
            #clone
        }
    }
}

fn gen_drop(drop_ext_fn_name: impl ToTokens, class_name: impl ToTokens + Display) -> TokenStream2 {
    let wrapper_fn_name = format_ident!("{class_name}_drop");
    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[unsafe(export_name = #drop_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name) {
            unsafe {
                if !_self.is_null() {
                    let _ = Box::from_raw(_self);
                }
            }
        }
    }
}

fn gen_clone(
    clone_ext_fn_name: impl ToTokens,
    class_name: impl ToTokens + Display,
) -> TokenStream2 {
    let wrapper_fn_name = format_ident!("{class_name}_clone");
    quote! {
        #[doc(hidden)]
        #[unsafe(export_name = #clone_ext_fn_name)]
        pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name) -> *mut #class_name {
            unsafe {
                let cloned: Box<#class_name> = Box::new((*_self).clone());
                Box::into_raw(cloned)
            }
        }
    }
}

fn gen_default_constructor(
    constructor_wrapper: &Option<DefaultConstructor>,
    class_name: impl ToTokens,
) -> TokenStream2 {
    if let Some(default_constructor) = constructor_wrapper {
        let extern_fn_name = &default_constructor.extern_fn_name;
        let constructor_name = &default_constructor.constructor_name;
        quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #constructor_name() -> *mut #class_name {
                unsafe {
                    let instance = Box::new(#class_name::default());
                    let ptr = Box::into_raw(instance);
                    ptr
                }
            }
        }
    } else {
        quote! {}
    }
}

fn map_primitive_field(
    FieldWrapper {
        field_name,
        field_type,
        getter,
        setter,
        ..
    }: &FieldWrapper,
    class_name: impl ToTokens + Display,
) -> TokenStream2 {
    let mut tokens = quote! {};
    if let Some(Getter {
        name,
        extern_fn_name,
    }) = getter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name) -> #field_type {
                unsafe {
                    (&*_self).#field_name
                }
            }
        })
    };
    if let Some(Setter {
        name,
        extern_fn_name,
    }) = setter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name, value: #field_type) {
                unsafe {
                    (&mut *_self).#field_name = value;
                }
            }
        });
    }
    tokens
}

fn map_string_field(
    FieldWrapper {
        field_name,
        getter,
        setter,
        ..
    }: &FieldWrapper,
    class_name: impl ToTokens + Display,
) -> TokenStream2 {
    let mut tokens = quote! {};
    if let Some(Getter {
        name,
        extern_fn_name,
    }) = getter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name) -> *mut FfiSlice {
                unsafe {
                    Box::into_raw(Box::new(FfiSlice {
                        ptr: (&*_self).#field_name.as_ptr(),
                        len: (&*_self).#field_name.len(),
                    }))
                }
            }
        });
    };
    if let Some(Setter {
        name,
        extern_fn_name,
    }) = setter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name, ptr: *const i8, _len: usize) {
                unsafe {
                    let s = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
                    (&mut *_self).#field_name = s;
                }
            }
        });
    }
    tokens
}

fn map_vec_field(
    FieldWrapper {
        setter,
        getter,
        field_type,
        field_name,
        ..
    }: &FieldWrapper,
    class_name: impl ToTokens + Display,
) -> TokenStream2 {
    let mut tokens = quote! {};

    if let Some(Getter {
        name,
        extern_fn_name,
    }) = getter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name) -> *mut #field_type {
                unsafe {
                    &mut(&mut *_self).#field_name as *mut #field_type
                }
            }
        });
    }

    if let Some(Setter {
        name,
        extern_fn_name,
    }) = setter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name, value: *mut #field_type) {
                unsafe {
                    std::mem::swap(&mut (*value), &mut (&mut *_self).#field_name);
                }
            }
        });
    }

    tokens
}

fn map_option_field(
    FieldWrapper {
        setter,
        getter,
        field_type,
        field_name,
        ..
    }: &FieldWrapper,
    class_name: impl ToTokens + Display,
) -> TokenStream2 {
    let mut tokens = quote! {};

    if let Some(Getter {
        name,
        extern_fn_name,
    }) = getter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name) -> *mut #field_type {
                unsafe {
                    &mut(&mut *_self).#field_name as *mut #field_type
                }
            }
        });
    }

    if let Some(Setter {
        name,
        extern_fn_name,
    }) = setter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name, value: *mut #field_type) {
                unsafe {
                    std::mem::swap(&mut (*value), &mut (&mut *_self).#field_name);
                }
            }
        });
    }

    tokens
}

fn map_custom_field(
    FieldWrapper {
        field_name,
        getter,
        setter,
        field_type,
        ..
    }: &FieldWrapper,
    class_name: impl ToTokens + Display,
) -> TokenStream2 {
    let mut tokens = quote! {};
    if let Some(Getter {
        name,
        extern_fn_name,
    }) = getter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name) -> *mut #field_type {
                unsafe {
                    Box::leak(Box::new((&*_self).#field_name.clone()))
                }
            }
        });
    };

    if let Some(Setter {
        name,
        extern_fn_name,
    }) = setter
    {
        let wrapper_fn_name = wrapper_fn_name(&class_name, name);
        tokens.extend(quote! {
            #[doc(hidden)]
            #[unsafe(export_name = #extern_fn_name)]
            pub unsafe extern "C" fn #wrapper_fn_name(_self: *mut #class_name, value: *mut #field_type) {
                unsafe {
                    (&mut *_self).#field_name = (*value).clone();
                }
            }
        });
    }

    tokens
}

fn wrapper_fn_name(class_name: impl Display, fn_name: impl Display) -> Ident {
    format_ident!("{class_name}_{fn_name}")
}

#[derive(Debug)]
pub struct DefaultConstructor {
    pub(crate) constructor_name: Ident,
    pub(crate) extern_fn_name: String,
}

pub struct FieldWrapper {
    pub(crate) field_name: Ident,
    pub(crate) field_type: Type,
    pub(crate) wrapper_type: WrapperType,
    pub(crate) setter: Option<Setter>,
    pub(crate) getter: Option<Getter>,
}

pub struct Getter {
    pub(crate) name: Ident,
    pub(crate) extern_fn_name: String,
}

pub struct Setter {
    pub(crate) name: Ident,
    pub(crate) extern_fn_name: String,
}
