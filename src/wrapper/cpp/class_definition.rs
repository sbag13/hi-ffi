use std::collections::HashSet;
use std::fmt::Display;

use quote::ToTokens;

use crate::prepend_each_line_with_n_tabs;

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

pub fn gen_class_source_from_impl_block(impl_block_wrapper: &ImplBlockWrapper) -> ClassSourceParts {
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
                ..
            } = map_args(m.args.iter());
            let return_types = map_return_type(&m.return_wrapper);
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
    ClassSourceParts {
        base: class_source_base(&impl_block_wrapper.struct_name),
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

    let pointer_constructor_definition = pointer_constructor_definition(class_name);
    let copy_constructor = copy_constructor_definition(struct_wrapper);
    let move_constructor = move_constructor_definition(struct_wrapper);

    let default_constructor = default_constructor(struct_wrapper);
    let default_constructor_definition = default_constructor.definition;

    let destructor = destructor(struct_wrapper);
    let destructor_definition = destructor.definition;

    let method_definitions = struct_wrapper
        .fields
        .iter()
        .map(|f| map_fields(f, &struct_wrapper.name))
        .fold(String::new(), |mut methods, Methods { getter, setter }| {
            if let Some(Method { definition, .. }) = getter {
                methods.push_str(&definition);
            }

            if let Some(Method { definition, .. }) = setter {
                methods.push_str(&definition);
            }

            methods
        });

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
        base: class_source_base(class_name),
        methods_definitions: definitions,
    }
}

fn class_source_base(class_name: impl Display) -> String {
    format!(
        r#"#include "{class_name}.h"

void* {class_name}::self_ptr() const {{
    return self;
}}
void {class_name}::set_self_ptr(void* ptr) {{
    self = ptr;
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

    let destructor = destructor(struct_wrapper);
    let destructor_declaration = destructor.declaration;
    let destructor_extern_fn = destructor.extern_fn;

    let pointer_constructor_declaration = pointer_constructor_declaration(class_name);
    let copy_constructor = copy_constructor_declaration(struct_wrapper);
    let move_constructor = move_constructor_declaration(struct_wrapper);
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
    void set_self_ptr(void* ptr);

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

fn move_constructor_definition(struct_wrapper: &StructWrapper) -> String {
    let class_name = &struct_wrapper.name;
    format!(
        r#"
{class_name}::{class_name}({class_name}&& other) {{
    this->self = other.self;
    other.self = nullptr;
}}"#,
    )
}
fn move_constructor_declaration(struct_wrapper: &StructWrapper) -> String {
    let class_name = &struct_wrapper.name;
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
            wrapper_type: FieldWrapperType::Primitive,
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
            wrapper_type: FieldWrapperType::String,
            setter,
            getter,
            ..
        } => {
            let getter = getter.as_ref().map(|g| map_string_getter(g, &class_name));
            let setter = setter.as_ref().map(|s| map_string_setter(s, &class_name));
            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: FieldWrapperType::Custom(_),
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
            wrapper_type: FieldWrapperType::Vec(inner),
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
            wrapper_type: FieldWrapperType::Option(inner),
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

fn destructor(struct_wrapper: &StructWrapper) -> Method {
    let class_name = &struct_wrapper.name;
    let drop_ext_fn_name = &struct_wrapper.drop_ext_fn_name;

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
