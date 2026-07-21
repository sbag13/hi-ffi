use std::collections::HashSet;
use std::fmt::Display;

use quote::ToTokens;

use crate::prepend_each_line_with_n_tabs;
use crate::wrapper::cpp::map_return_type;

use super::*;

pub const INCLUDES_MARKER: &str = "// includes";
pub const EXTERN_FNS_MARKER: &str = "// extern fns";
pub const METHOD_DEFINITIONS_MARKER: &str = "// methods definitions";

pub struct ClassHeaderParts {
    pub class_definition: String,
    pub includes: HashSet<String>,
    pub extern_fns: String,
    pub method_declarations: String,
}

pub fn gen_trait_vtable(trait_wrapper: &TraitWrapper) -> String {
    let mut vtable_functions = trait_wrapper
        .functions
        .iter()
        .map(|method| {
            let ext_method_name = &method.extern_function_name;

            let MappedCppFunctionArgsTokens { wrapper_args, .. } = map_args(method.args.iter());
            let wrapper_args = if wrapper_args.is_empty() {
                wrapper_args
            } else {
                format!(", {wrapper_args}")
            };

            let return_type = trait_bridge_fn_ret_type(&method.return_wrapper);

            format!("    {return_type} (*{ext_method_name})(void* self{wrapper_args});\n",)
        })
        .collect::<String>();
    vtable_functions.pop(); // remove last comma and newline

    let class_name = &trait_wrapper.name;

    format!(
        r#"struct {class_name}VTable {{
{vtable_functions}
}};"#
    )
}

pub fn gen_interface_class(trait_wrapper: &TraitWrapper) -> ClassHeaderParts {
    let class_name = &trait_wrapper.name;

    let (abstract_method_declarations, mut includes, extern_fns_decl, overridden_methods) =
        trait_wrapper.functions.iter().fold(
            (String::new(), HashSet::new(), String::new(), String::new()),
            |(mut methods, mut includes, mut extern_fns_decl, mut overridden_methods), method| {
                let MappedCppFunctionArgsTokens {
                    cpp_args,
                    wrapper_args,
                    includes: arg_includes,
                    ..
                } = map_args(method.args.iter());

                let return_type = trait_bridge_fn_ret_type(&method.return_wrapper);
                let cpp_return_type = map_return_type(&method.return_wrapper).return_type;

                let ext_method_name = &method.extern_function_name;
                let method_name = &method.name;

                let abstract_method_declaration = format!(
                    r#"    virtual {cpp_return_type} {method_name}({cpp_args}) = 0;
"#,
                );
                let method_declaration = format!(
                    r#"    virtual {cpp_return_type} {method_name}({cpp_args}) override;
"#,
                );

                let wrapper_args = if wrapper_args.is_empty() {
                    wrapper_args
                } else {
                    format!(", {wrapper_args}")
                };
                let extern_fn_decl = format!(
                    r#"    {return_type} {ext_method_name}(void* self{wrapper_args});
"#,
                );

                methods.push_str(&abstract_method_declaration);
                includes.extend(arg_includes);
                extern_fns_decl.push_str(&extern_fn_decl);
                overridden_methods.push_str(&method_declaration);

                (methods, includes, extern_fns_decl, overridden_methods)
            },
        );

    let vtable = prepend_each_line_with_n_tabs(gen_trait_vtable(trait_wrapper).as_str(), 1);
    let class_name_uppercase = class_name.to_string().to_uppercase();
    let mut vtable_fields = trait_wrapper
        .functions
        .iter()
        .map(|method| {
            let ext_method_name = &method.extern_function_name;
            format!("    {ext_method_name},\n",)
        })
        .collect::<String>();
    vtable_fields.pop(); // remove last comma and newline

    let rust_impl_class_name = format!("{class_name}RustImpl");
    let impl_pointer_constructor = pointer_constructor_declaration(rust_impl_class_name.as_str());
    let impl_destructor_decl = format!("    virtual ~{rust_impl_class_name}();\n");
    let impl_move_constructor = move_constructor_declaration(rust_impl_class_name.as_str());

    let mut box_dyn_ext_functions =
        trait_wrapper
            .functions
            .iter()
            .fold(String::new(), |mut acc, method| {
                let MappedCppFunctionArgsTokens { wrapper_args, .. } = map_args(method.args.iter());

                let extern_fn = method_extern_fn(
                    format!("{}_BoxDyn", method.extern_function_name),
                    false,
                    &wrapper_args,
                    &map_return_type(&method.return_wrapper).ext_return_type,
                );
                acc.push_str(&extern_fn);
                acc
            });

    includes.insert("#include <memory>\n".to_string());

    let box_dyn_destructor_ext_fn =
        format!("\n    void {EXPORTED_SYMBOLS_PREFIX}{class_name}_BoxDyn_drop(void* self);");
    box_dyn_ext_functions.push_str(&box_dyn_destructor_ext_fn);

    ClassHeaderParts {
        class_definition: format!(
            r#"
#ifndef {class_name}__def
#define {class_name}__def

#include "base.h"

{INCLUDES_MARKER}

// Class definition
class {class_name} {{
public:
    virtual ~{class_name}() = default;

    {METHOD_DEFINITIONS_MARKER}
}};

extern "C" {{
{EXTERN_FNS_MARKER}
{vtable}

    static const {class_name}VTable {class_name_uppercase}_VTABLE_INST = {{
{vtable_fields}
    }};

    struct {class_name}Bridge {{
        void* obj;
        const {class_name}VTable* vtable;
        void (*deleter)(void*);
    }};
}}

extern "C" {{{box_dyn_ext_functions}
}}

class {rust_impl_class_name} : public {class_name} {{
    void* self = nullptr;
public:

    void* self_ptr() const;
    void* leak();

{impl_pointer_constructor}
{impl_move_constructor}
{impl_destructor_decl}
{overridden_methods}
}};

#endif
"#
        ),
        includes,
        extern_fns: extern_fns_decl,
        method_declarations: abstract_method_declarations,
    }
}

pub fn gen_trait_methods_definitions(trait_wrapper: &TraitWrapper) -> ClassSourceParts {
    let class_name = &trait_wrapper.name;
    let rust_impl_class_name = format!("{}RustImpl", trait_wrapper.name);

    let mut source_includes = HashSet::new();
    let mut methods_definitions = trait_wrapper.functions.iter().fold(
        String::new(),
        |mut methods_definitions, method| {
            let return_type = trait_bridge_fn_ret_type(&method.return_wrapper);

            let method_name = &method.name;
            let ext_method_name = &method.extern_function_name;

            let MappedCppFunctionArgsTokens {
                call_args,
                wrapper_args,
                from_rust_casts,
                source_includes: method_source_includes,
                ..
            } = map_args(method.args.iter());

            let wrapper_args = if wrapper_args.is_empty() {
                wrapper_args
            } else {
                format!(", {wrapper_args}")
            };

            let from_rust_casts = prepend_each_line_with_n_tabs(&from_rust_casts, 1);

            let get_and_return_result = match &method.return_wrapper {
                Some(rt) => {
                    match &rt.wrapper_type {
                        WrapperType::Vec(inner) => {
                            let inner_name = inner.name();
                            format!("auto cpp_vec = (*static_cast<std::shared_ptr<{class_name}>*>(self))->{method_name}({call_args});
auto rust_vec = Rust{inner_name}Vec::from_std(cpp_vec);
return rust_vec.leak();")
                        }

                        WrapperType::String => {
                            format!("std::string cpp_string = (*static_cast<std::shared_ptr<{class_name}>*>(self))->{method_name}({call_args});
auto rust_string_ptr = {RUST_STRING_FROM_C_PTR_FN_NAME}(cpp_string.data());
return rust_string_ptr;"
                )
                        }

                        WrapperType::Option(inner) => {
                            let inner_name = inner.name();
                            format!("auto cpp_option = (*static_cast<std::shared_ptr<{class_name}>*>(self))->{method_name}({call_args});
auto rust_option = Rust{inner_name}Option::from_std(cpp_option);
return rust_option.leak();")
                        }

                        WrapperType::Struct(_) => {
                            format!("return (*static_cast<std::shared_ptr<{class_name}>*>(self))->{method_name}({call_args}).leak();")
                        },

                        _ => {
                            format!("return (*static_cast<std::shared_ptr<{class_name}>*>(self))->{method_name}({call_args});")
                        }
                    }
                }


                _ => {
                    format!("return (*static_cast<std::shared_ptr<{class_name}>*>(self))->{method_name}({call_args});")
                }
            };
            let get_and_return_result = prepend_each_line_with_n_tabs(&get_and_return_result, 1);

            let extern_fn_decl = format!(
                r#"{return_type} {ext_method_name}(void* self{wrapper_args}) {{
{from_rust_casts}
{get_and_return_result}
}};
"#,
            );

            let extern_fn = format!("{}_BoxDyn", method.extern_function_name);
            let MappedCppFunctionArgsTokens {
                cpp_args,
                call_args,
                arg_casts,
                ..
            } = map_args(method.args.iter());
            let return_types = map_return_type(&method.return_wrapper);
            let return_type_includes = return_types.return_type_includes.clone();
            source_includes.extend(method_source_includes);
            source_includes.extend(return_type_includes);
            let rust_impl_method = method_definition(&method.name, extern_fn, false, &cpp_args, &call_args, &arg_casts, &return_types, &rust_impl_class_name);

            methods_definitions.push_str(&format!("{extern_fn_decl}
{rust_impl_method}
"));

            methods_definitions
        },
    );

    let rust_impl_constructor = pointer_constructor_definition(&rust_impl_class_name);
    let rust_impl_destructor = destructor(
        &rust_impl_class_name,
        format!("{EXPORTED_SYMBOLS_PREFIX}{}_BoxDyn_drop", class_name),
    );
    let rust_impl_move_constructor = move_constructor_definition(&rust_impl_class_name);
    methods_definitions.extend([
        rust_impl_constructor.as_str(),
        rust_impl_destructor.definition.as_str(),
        rust_impl_move_constructor.as_str(),
    ]);

    let source_includes = source_includes
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");

    ClassSourceParts {
        base: format!(r#"#include "{class_name}.h"
{source_includes}"#),
        methods_definitions,
    }
}

fn trait_bridge_fn_ret_type(ret_wrapper: &Option<FunctionReturnWrapper>) -> String {
    match &ret_wrapper {
        Some(ret) => match &ret.wrapper_type {
            WrapperType::IntegerNumber(ty)
            | WrapperType::FloatingPointNumber(ty)
            | WrapperType::Enum(ty) => ty.to_string(),
            WrapperType::Bool => "bool".to_string(),
            WrapperType::String
            | WrapperType::Struct(_)
            | WrapperType::Vec(_)
            | WrapperType::Option(_)
            | WrapperType::Result(_) => "void*".to_string(),
            WrapperType::UnitExpr => "void".to_string(),
            WrapperType::Trait(_) => {
                panic!("Trait return type is not supported in traits")
            }
        },
        None => "void".to_string(),
    }
}

pub fn gen_class_source_from_impl_block(impl_block_wrapper: &ImplBlockWrapper) -> ClassSourceParts {
    let mut source_includes = HashSet::new();
    let methods_definitions = impl_block_wrapper
        .methods
        .iter()
        .filter_map(|m| {
            if !m.public {
                return None;
            }
            let MappedCppFunctionArgsTokens {
                cpp_args,
                call_args,
                arg_casts,
                source_includes: method_source_includes,
                ..
            } = map_args(m.args.iter());
            let return_types = map_return_type(&m.return_wrapper);
            let return_type_includes = return_types.return_type_includes.clone();
            source_includes.extend(method_source_includes);
            source_includes.extend(return_type_includes);
            Some(method_definition(
                &m.name,
                &m.extern_function_name,
                m.is_static,
                &cpp_args,
                &call_args,
                &arg_casts,
                &return_types,
                &impl_block_wrapper.struct_name.to_string(),
            ))
        })
        .collect();
    let source_includes = source_includes
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");

    ClassSourceParts {
        base: format!(r#"{source_includes}

{}"#, class_source_base(&impl_block_wrapper.struct_name)),
        methods_definitions,
    }
}

pub fn gen_class_definition_parts_from_impl_block(
    impl_block_wrapper: &ImplBlockWrapper,
) -> ClassHeaderParts {
    let (method_declarations, extern_fns, includes) = impl_block_wrapper
        .methods
        .iter()
        .filter(|method| method.public)
        .fold(
            (String::new(), String::new(), HashSet::new()),
            |(mut methods, mut externs, mut includes), method_wrapper| {
                let method_name = &method_wrapper.name;
                let extern_function_name = &method_wrapper.extern_function_name;

                let MappedCppFunctionArgsTokens {
                    cpp_args,
                    wrapper_args,
                    includes: arg_includes,
                    ..
                } = map_args(method_wrapper.args.iter());

                let return_types = map_return_type(&method_wrapper.return_wrapper);

                let method = method_declaration(
                    method_name,
                    method_wrapper.is_static,
                    &cpp_args,
                    &return_types,
                );
                let extern_fn = method_extern_fn(
                    extern_function_name,
                    method_wrapper.is_static,
                    &wrapper_args,
                    &return_types.ext_return_type,
                );

                methods.push_str(&method);
                externs.push_str(&extern_fn);
                // Add includes from arguments
                includes.extend(arg_includes);
                // Add includes from return type
                includes.extend(return_types.return_type_includes);

                (methods, externs, includes)
            },
        );

    ClassHeaderParts {
        class_definition: gen_empty_class_definition(&impl_block_wrapper.struct_name),
        includes,
        extern_fns: format!(
            r#"
extern "C" {{
{extern_fns}
}}
"#
        ),
        method_declarations,
    }
}

fn method_extern_fn(
    extern_function_name: impl Display,
    is_static: bool,
    wrapper_args: &str,
    ext_return_type: &str,
) -> String {
    let wrapper_args = if !is_static {
        if !wrapper_args.is_empty() {
            format!("void* self, {}", wrapper_args)
        } else {
            "void* self".to_string()
        }
    } else {
        wrapper_args.to_string()
    };

    format!(
        r#"
    {ext_return_type} {extern_function_name}({wrapper_args});"#
    )
}

pub struct ClassSourceParts {
    /// inserted one time only per file
    pub base: String,
    pub methods_definitions: String,
}

fn method_declaration(
    method_name: impl Display,
    is_static: bool,
    cpp_args: &str,
    return_types: &ReturnTypes,
) -> String {
    let static_keyword = if is_static { "static " } else { "" };

    let ReturnTypes { return_type, .. } = return_types;

    format!(
        r#"    {static_keyword}{return_type} {method_name}({cpp_args});
"#
    )
}

#[allow(clippy::too_many_arguments)]
fn method_definition(
    method_name: impl Display,
    extern_function_name: impl Display,
    is_static: bool,
    cpp_args: &str,
    arg_names: &str,
    arg_casts: &str,
    return_types: &ReturnTypes,
    class_name: &str,
) -> String {
    let arg_names = if is_static {
        arg_names.to_string()
    } else if arg_names.is_empty() {
        "this->self".to_string()
    } else {
        format!("this->self, {}", arg_names)
    };

    let ReturnTypes {
        ext_return_type,
        return_type,
        return_cast,
        ..
    } = return_types;

    let arg_casts = prepend_each_line_with_n_tabs(arg_casts, 1);
    let return_casts = prepend_each_line_with_n_tabs(return_cast, 1);

    format!(
        r#"
{return_type} {class_name}::{method_name}({cpp_args}) {{
{arg_casts}
    {ext_return_type} result = {extern_function_name}({arg_names});
{return_casts}
}}
"#
    )
}

pub fn gen_methods_definitions_from_struct(struct_wrapper: &StructWrapper) -> ClassSourceParts {
    let class_name = &struct_wrapper.name;
    let mut source_includes = HashSet::new();

    let pointer_constructor_definition = pointer_constructor_definition(class_name);
    let copy_constructor = copy_constructor_definition(struct_wrapper);
    let move_constructor = move_constructor_definition(&struct_wrapper.name);

    let default_constructor = default_constructor(struct_wrapper);
    let default_constructor_definition = default_constructor.definition;

    let destructor = destructor(&struct_wrapper.name, &struct_wrapper.drop_ext_fn_name);
    let destructor_definition = destructor.definition;

    let method_definitions = struct_wrapper
        .fields
        .iter()
        .map(|f| map_fields(f, &struct_wrapper.name))
        .fold(String::new(), |mut methods, Methods { getter, setter }| {
            if let Some(Method { definition, include, .. }) = getter {
                methods.push_str(&definition);
                if !include.is_empty() {
                    source_includes.insert(include);
                }
            }

            if let Some(Method { definition, include, .. }) = setter {
                methods.push_str(&definition);
                if !include.is_empty() {
                    source_includes.insert(include);
                }
            }

            methods
        });

    let source_includes = source_includes
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");

    let definitions = format!(
        r#"
{pointer_constructor_definition}
{copy_constructor}
{move_constructor}
{default_constructor_definition}
{destructor_definition}
{method_definitions}
"#
    );

    ClassSourceParts {
        base: format!(r#"{source_includes}

{}"#, class_source_base(class_name)),
        methods_definitions: definitions,
    }
}

fn class_source_base(class_name: impl Display) -> String {
    format!(
        r#"#include "{class_name}.h"

void* {class_name}::self_ptr() const {{
    return self;
}}
void* {class_name}::leak() {{
    void* leaked_self = self;
    self = nullptr;
    return leaked_self;
}}
"#
    )
}

pub fn gen_class_definition_parts_from_struct(struct_wrapper: &StructWrapper) -> ClassHeaderParts {
    let class_name = &struct_wrapper.name;
    let (method_declarations, extern_fns, includes) = struct_wrapper
        .fields
        .iter()
        .map(|f| map_fields(f, &struct_wrapper.name))
        .fold(
            (String::new(), String::new(), HashSet::new()),
            |(mut methods, mut externs, mut includes), Methods { getter, setter }| {
                if let Some(Method {
                    declaration,
                    extern_fn,
                    include,
                    ..
                }) = getter
                {
                    methods.push_str(&declaration);
                    externs.push_str(&extern_fn);
                    includes.insert(include);
                }

                if let Some(Method {
                    declaration,
                    extern_fn,
                    include,
                    ..
                }) = setter
                {
                    methods.push_str(&declaration);
                    externs.push_str(&extern_fn);
                    includes.insert(include);
                }

                (methods, externs, includes)
            },
        );
    let default_constructor = default_constructor(struct_wrapper);
    let default_constructor_declaration = default_constructor.declaration;
    let default_constructor_extern_fn = default_constructor.extern_fn;

    let destructor = destructor(&struct_wrapper.name, &struct_wrapper.drop_ext_fn_name);
    let destructor_declaration = destructor.declaration;
    let destructor_extern_fn = destructor.extern_fn;

    let pointer_constructor_declaration = pointer_constructor_declaration(class_name);
    let copy_constructor = copy_constructor_declaration(struct_wrapper);
    let move_constructor = move_constructor_declaration(&struct_wrapper.name);
    let clone_extern_fn = clone_ext_fn(struct_wrapper);

    ClassHeaderParts {
        class_definition: gen_empty_class_definition(class_name),
        includes,
        extern_fns: format!(
            r#"
extern "C" {{
{extern_fns}
{default_constructor_extern_fn}
{destructor_extern_fn}
{clone_extern_fn}
}}
"#
        ),
        method_declarations: format!(
            r#"
{pointer_constructor_declaration}
{copy_constructor}
{move_constructor}
{default_constructor_declaration}
{destructor_declaration}
{method_declarations}
"#
        ),
    }
}

fn gen_empty_class_definition(class_name: impl Display) -> String {
    format!(
        r#"
#ifndef {class_name}__def
#define {class_name}__def

#include "base.h"

{INCLUDES_MARKER}
{EXTERN_FNS_MARKER}

// Class definition
class {class_name} {{
    void* self = nullptr;
public:

    void* self_ptr() const;
    void* leak();

    {METHOD_DEFINITIONS_MARKER}
}};

#endif
"#
    )
}

fn clone_ext_fn(struct_wrapper: &StructWrapper) -> String {
    let clone_ext_fn_name = &struct_wrapper.clone_ext_fn_name;
    format!("    void* {clone_ext_fn_name}(void*);\n",)
}

fn copy_constructor_definition(struct_wrapper: &StructWrapper) -> String {
    let class_name = &struct_wrapper.name;
    let clone_ext_fn_name = &struct_wrapper.clone_ext_fn_name;
    format!(
        r#"
{class_name}::{class_name}(const {class_name}& other) {{
    this->self = {clone_ext_fn_name}(other.self);
}}"#,
    )
}
fn copy_constructor_declaration(struct_wrapper: &StructWrapper) -> String {
    let class_name = &struct_wrapper.name;
    format!(r#"    {class_name}(const {class_name}& other);"#,)
}

fn move_constructor_definition(class_name: impl Display) -> String {
    format!(
        r#"
{class_name}::{class_name}({class_name}&& other) {{
    this->self = other.self;
    other.self = nullptr;
}}"#,
    )
}
fn move_constructor_declaration(class_name: impl Display) -> String {
    format!(
        r#"
    {class_name}({class_name}&& other);"#,
    )
}

fn pointer_constructor_definition(class_name: impl Display) -> String {
    format!(r#"{class_name}::{class_name}(void* self) : self(self) {{}}"#)
}
fn pointer_constructor_declaration(class_name: impl Display) -> String {
    format!(r#"    {class_name}(void* self);"#)
}

fn map_fields(field: &FieldWrapper, class_name: impl Display) -> Methods {
    match field {
        FieldWrapper {
            field_type,
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool,
            setter,
            getter,
            ..
        } => {
            let field_type = quote::quote! { #field_type }.to_string();

            let getter = getter
                .as_ref()
                .map(|g| map_primitive_getter(g, &field_type, &class_name));
            let setter = setter
                .as_ref()
                .map(|s| map_primitive_setter(s, &field_type, &class_name));

            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: WrapperType::Enum(_),
            setter,
            getter,
            field_type,
            ..
        } => {
            let field_type = quote::quote! { #field_type }.to_string();
            let mut getter = getter
                .as_ref()
                .map(|g| map_primitive_getter(g, &field_type, &class_name));
            if let Some(g) = getter
                .as_mut() { g.include.push_str(&format!("#include \"{field_type}.h\"")) } // Enums are represented as their underlying integer type in C++
            let setter = setter
                .as_ref()
                .map(|s| map_primitive_setter(s, &field_type, &class_name));

            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: WrapperType::String,
            setter,
            getter,
            ..
        } => {
            let getter = getter.as_ref().map(|g| map_string_getter(g, &class_name));
            let setter = setter.as_ref().map(|s| map_string_setter(s, &class_name));
            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: WrapperType::Struct(_),
            setter,
            getter,
            field_type,
            ..
        } => {
            let getter = getter.as_ref().map(|g| {
                map_custom_getter(g, field_type.to_token_stream().to_string(), &class_name)
            });
            let setter = setter.as_ref().map(|s| {
                map_custom_setter(s, field_type.to_token_stream().to_string(), &class_name)
            });
            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: WrapperType::Vec(inner),
            getter,
            setter,
            ..
        } => {
            let getter = getter
                .as_ref()
                .map(|g| map_vec_getter(g, inner, &class_name));
            let setter = setter
                .as_ref()
                .map(|s| map_vec_setter(s, inner, &class_name));

            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: WrapperType::Option(inner),
            getter,
            setter,
            ..
        } => {
            let getter = getter
                .as_ref()
                .map(|g| map_option_getter(g, inner, &class_name));
            let setter = setter
                .as_ref()
                .map(|s| map_option_setter(s, inner, &class_name));

            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: WrapperType::Result(_),
            ..
        } => {
            panic!("Result wrapper type is not yet supported for struct fields")
        }

        FieldWrapper {
            wrapper_type: WrapperType::Trait(_),
            ..
        } => {
            panic!("Trait wrapper type is not yet supported for struct fields")
        }

        FieldWrapper {
            wrapper_type: WrapperType::UnitExpr,
            ..
        } => {
            panic!("UnitExpr wrapper type is not supported for struct fields")
        }
    }
}

fn map_vec_getter(
    Getter {
        name,
        extern_fn_name,
    }: &Getter,
    inner: &WrapperType,
    class_name: impl Display,
) -> Method {
    let inner_name = inner.name();
    let cpp_inner_name = match inner {
        WrapperType::String => "std::string",
        _ => inner_name.as_str(),
    };
    let cpp_vec_file_name = format!("vec_{inner_name}");
    let cpp_vec_class_name = format!("Rust{inner_name}Vec");

    Method {
        declaration: format!("    std::vector<{cpp_inner_name}> {name}();\n"),
        definition: format!(
            r#"
std::vector<{cpp_inner_name}> {class_name}::{name}() {{
    void* result = {extern_fn_name}(this->self);
    auto rust_vec = {cpp_vec_class_name}(result);
    auto std_vec = rust_vec.to_std();
    // this vec is still owned by a Rust struct - avoid calling drop by cpp
    rust_vec.leak();
    return std_vec;
}}
"#
        ),
        extern_fn: format!("    void* {extern_fn_name}(void*);\n"),
        include: custom_type_include(cpp_vec_file_name),
    }
}

fn map_vec_setter(
    Setter {
        name,
        extern_fn_name,
    }: &Setter,
    inner: &WrapperType,
    class_name: impl Display,
) -> Method {
    let inner_name = inner.name();
    let cpp_inner_name = match inner {
        WrapperType::String => "std::string",
        _ => inner_name.as_str(),
    };

    let cpp_vec_file_name = format!("vec_{inner_name}");
    let cpp_vec_class_name = format!("Rust{inner_name}Vec");

    Method {
        declaration: format!("    void {name}(std::vector<{cpp_inner_name}>& value);\n"),
        definition: format!(
            r#"
void {class_name}::{name}(std::vector<{cpp_inner_name}>& value) {{
    auto rust_vec = {cpp_vec_class_name}::from_std(value);
    {extern_fn_name}(this->self, rust_vec.raw_ptr()); // Rust side makes swap
}}"#
        ),
        extern_fn: format!("    void {extern_fn_name}(void*, void*);\n"),
        include: custom_type_include(cpp_vec_file_name),
    }
}

fn map_option_getter(
    Getter {
        name,
        extern_fn_name,
    }: &Getter,
    inner: &WrapperType,
    class_name: impl Display,
) -> Method {
    let inner_name = inner.name();
    let cpp_option_class_name = format!("Rust{inner_name}Option");
    let cpp_option_file_name = format!("option_{inner_name}");
    let cpp_inner_type = cpp_type(inner);
    let mut includes = custom_type_include(&cpp_option_file_name);
    includes.push_str("#include <optional>\n");

    Method {
        declaration: format!("    std::optional<{cpp_inner_type}> {name}();\n"),
        definition: format!(
            r#"
std::optional<{cpp_inner_type}> {class_name}::{name}() {{
    void* result = {extern_fn_name}(this->self);
    auto rust_option = {cpp_option_class_name}(result);
    auto value = rust_option.to_std();
    rust_option.leak();
    return value;
}}
"#
        ),
        extern_fn: format!("    void* {extern_fn_name}(void*);\n"),
        include: includes,
    }
}

fn map_option_setter(
    Setter {
        name,
        extern_fn_name,
    }: &Setter,
    inner: &WrapperType,
    class_name: impl Display,
) -> Method {
    let inner_name = inner.name();
    let cpp_option_file_name = format!("option_{inner_name}");
    let cpp_inner_type = cpp_type(inner);
    let mut includes = custom_type_include(&cpp_option_file_name);
    includes.push_str("#include <optional>\n");

    Method {
        declaration: format!("    void {name}(const std::optional<{cpp_inner_type}>& value);\n"),
        definition: format!(
            r#"
void {class_name}::{name}(const std::optional<{cpp_inner_type}>& value) {{
    auto rust_option = Rust{inner_name}Option::from_std(value);
    {extern_fn_name}(this->self, rust_option.raw_ptr());
}}"#
        ),
        extern_fn: format!("    void {extern_fn_name}(void*, void*);\n"),
        include: includes,
    }
}

fn map_custom_getter(
    Getter {
        name,
        extern_fn_name,
    }: &Getter,
    field_type: impl Display,
    class_name: impl Display,
) -> Method {
    Method {
        declaration: format!("    {field_type} {name}();\n"),
        definition: format!(
            r#"
{field_type} {class_name}::{name}() {{
    return {field_type}({extern_fn_name}(this->self));
}}"#
        ),
        extern_fn: format!("    void* {extern_fn_name}(void*);\n"),
        include: custom_type_include(field_type),
    }
}

fn map_custom_setter(
    Setter {
        name,
        extern_fn_name,
    }: &Setter,
    field_type: impl Display,
    class_name: impl Display,
) -> Method {
    Method {
        declaration: format!("    void {name}({field_type}& value);\n"),
        definition: format!(
            r#"
void {class_name}::{name}({field_type}& value) {{
    {extern_fn_name}(this->self, value.self_ptr()); // Rust side makes clone
}}"#
        ),
        extern_fn: format!("    void {extern_fn_name}(void*, void*);\n"),
        include: custom_type_include(field_type),
    }
}

fn custom_type_include(field_type: impl Display) -> String {
    format!("#include \"{field_type}.h\"\n")
}

fn map_string_getter(
    Getter {
        name,
        extern_fn_name,
    }: &Getter,
    class_name: impl Display,
) -> Method {
    Method {
        declaration: format!("    std::string {name}();\n"),
        definition: format!(
            r#"
std::string {class_name}::{name}() {{
    void* slice_ptr = {extern_fn_name}(this->self);
    char* ptr = {SLICE_GET_PTR_FN_NAME}(slice_ptr);
    auto len = {SLICE_GET_LEN_FN_NAME}(slice_ptr);
    auto result = std::string(ptr, len);
    {SLICE_DROP_FN_NAME}(slice_ptr);
    return result;
}}"#
        ),
        extern_fn: format!("    void* {extern_fn_name}(void*);\n"),
        include: String::new(),
    }
}

fn map_string_setter(
    Setter {
        name,
        extern_fn_name,
    }: &Setter,
    class_name: impl Display,
) -> Method {
    Method {
        declaration: format!(
            "    void {name}(std::string&& value);
    void {name}(std::string& value);\n"
        ),
        definition: format!(
            r#"
void {class_name}::{name}(std::string&& value) {{
    auto ptr = value.data();
    auto len = value.size();
    {extern_fn_name}(this->self, ptr, len);
}}

void {class_name}::{name}(std::string& value) {{
    this->{name}(std::move(value));
}}"#
        ),
        extern_fn: format!("    void {extern_fn_name}(void*, const char*, size_t);\n"),
        include: String::new(),
    }
}

fn map_primitive_getter(
    Getter {
        name,
        extern_fn_name,
    }: &Getter,
    field_type: impl Display,
    class_name: impl Display,
) -> Method {
    Method {
        declaration: format!("    {field_type} {name}();\n"),
        definition: format!(
            r#"
{field_type} {class_name}::{name}() {{
    return {field_type}({extern_fn_name}(this->self));
}}"#
        ),
        extern_fn: format!("    {field_type} {extern_fn_name}(void*);\n"),
        include: String::new(),
    }
}

fn map_primitive_setter(
    Setter {
        name,
        extern_fn_name,
    }: &Setter,
    field_type: impl Display,
    class_name: impl Display,
) -> Method {
    Method {
        declaration: format!("    void {name}({field_type} value);\n"),
        definition: format!(
            r#"
void {class_name}::{name}({field_type} value) {{
    {extern_fn_name}(this->self, value);
}}"#
        ),
        extern_fn: format!("    void {extern_fn_name}(void*, {field_type});\n"),
        include: String::new(),
    }
}

fn default_constructor(struct_wrapper: &StructWrapper) -> Method {
    let class_name = &struct_wrapper.name;

    if let Some(default_constructor) = struct_wrapper.default_constructor.as_ref() {
        let default_constructor_ext_fn_name = &default_constructor.extern_fn_name;

        let definition = format!(
            r#"
{class_name}::{class_name}() {{
    this->self = {default_constructor_ext_fn_name}();
}}
"#,
        );

        let extern_fn = format!("    void* {default_constructor_ext_fn_name}();");

        Method {
            declaration: format!("    {class_name}();"),
            definition,
            extern_fn,
            include: String::new(),
        }
    } else {
        Method {
            declaration: String::new(),
            definition: String::new(),
            extern_fn: String::new(),
            include: String::new(),
        }
    }
}

fn destructor(class_name: impl Display, drop_ext_fn_name: impl Display) -> Method {
    let definition = format!(
        r#"
{class_name}::~{class_name}() {{
    if (this->self != nullptr)
        {drop_ext_fn_name}(this->self);
}}"#,
    );

    let extern_fn = format!("    void {drop_ext_fn_name}(void*);");

    Method {
        declaration: format!("    virtual ~{class_name}();\n"),
        definition,
        extern_fn,
        include: String::new(),
    }
}
