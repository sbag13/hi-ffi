use std::{collections::HashSet, fmt::Display};

use quote::ToTokens;

use super::*;

pub fn gen_class_definition(struct_wrapper: &StructWrapper) -> String {
    let class_name = &struct_wrapper.name;
    let (method_definitions, extern_fns, includes) =
        struct_wrapper.fields.iter().map(map_fields).fold(
            (String::new(), String::new(), HashSet::new()),
            |(mut methods, mut externs, mut includes), Methods { getter, setter }| {
                if let Some(Method {
                    definition,
                    extern_fn,
                    include,
                }) = getter
                {
                    methods.push_str(&definition);
                    externs.push_str(&extern_fn);
                    includes.insert(include);
                }

                if let Some(Method {
                    definition,
                    extern_fn,
                    include,
                }) = setter
                {
                    methods.push_str(&definition);
                    externs.push_str(&extern_fn);
                    includes.insert(include);
                }

                (methods, externs, includes)
            },
        );
    let default_constructor = default_constructor(struct_wrapper);
    let default_constructor_definition = default_constructor.definition;
    let default_constructor_extern_fn = default_constructor.extern_fn;

    let destructor = destructor(struct_wrapper);
    let destructor_definition = destructor.definition;
    let destructor_extern_fn = destructor.extern_fn;

    let includes = includes.into_iter().fold(String::new(), |mut acc, i| {
        acc.push_str(&i);
        acc
    });

    let pointer_constructor_definition = pointer_constructor_definition(class_name);
    let copy_constructor = copy_constructor_definition(struct_wrapper);
    let move_constructor = move_constructor_definition(struct_wrapper);
    let clone_extern_fn = clone_ext_fn(struct_wrapper);

    format!(
        r#"
#ifndef {class_name}__def
#define {class_name}__def

#include "base.h"
{includes}

extern "C" {{
{extern_fns}
{default_constructor_extern_fn}
{destructor_extern_fn}
{clone_extern_fn}
}}

class {class_name} {{
    void* self = nullptr;
public:
{pointer_constructor_definition}
{copy_constructor}
{move_constructor}
{default_constructor_definition}
{destructor_definition}
{method_definitions}

    void* self_ptr() {{
        return self;
    }}
    void set_self_ptr(void* ptr) {{
        self = ptr;
    }}
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
    {class_name}(const {class_name}& other) {{
        this->self = {clone_ext_fn_name}(other.self);
    }}"#,
    )
}

fn move_constructor_definition(struct_wrapper: &StructWrapper) -> String {
    let class_name = &struct_wrapper.name;
    format!(
        r#"
    {class_name}({class_name}&& other) {{
        this->self = other.self;
        other.self = nullptr;
    }}"#,
    )
}

fn pointer_constructor_definition(class_name: impl Display) -> String {
    format!(r#"    {class_name}(void* self) : self(self) {{}}"#,)
}

fn map_fields(field: &FieldWrapper) -> Methods {
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
                .map(|g| map_primitive_getter(g, &field_type));
            let setter = setter
                .as_ref()
                .map(|s| map_primitive_setter(s, &field_type));

            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: FieldWrapperType::String,
            setter,
            getter,
            ..
        } => {
            let getter = getter.as_ref().map(map_string_getter);
            let setter = setter.as_ref().map(map_string_setter);
            Methods { getter, setter }
        }

        FieldWrapper {
            wrapper_type: FieldWrapperType::Custom,
            setter,
            getter,
            field_type,
            ..
        } => {
            let getter = getter
                .as_ref()
                .map(|g| map_custom_getter(g, field_type.to_token_stream().to_string()));
            let setter = setter
                .as_ref()
                .map(|s| map_custom_setter(s, field_type.to_token_stream().to_string()));
            Methods { getter, setter }
        }
    }
}

fn map_custom_getter(
    Getter {
        name,
        extern_fn_name,
    }: &Getter,
    field_type: impl Display,
) -> Method {
    Method {
        definition: format!(
            r#"
    {field_type} {name}() {{
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
) -> Method {
    Method {
        definition: format!(
            r#"
    void {name}({field_type}& value) {{
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
) -> Method {
    Method {
        definition: format!(
            r#"
    std::string {name}() {{
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
) -> Method {
    Method {
        definition: format!(
            r#"
    void {name}(std::string&& value) {{
        auto ptr = value.data();
        auto len = value.size();
        {extern_fn_name}(this->self, ptr, len);
    }}
    
    void {name}(std::string& value) {{
        this->{name}(std::move(value));
    }}"#
        ),
        extern_fn: format!("    void {extern_fn_name}(void*, char*, size_t);\n"),
        include: String::new(),
    }
}

fn map_primitive_getter(
    Getter {
        name,
        extern_fn_name,
    }: &Getter,
    field_type: impl Display,
) -> Method {
    Method {
        definition: format!(
            r#"
    {field_type} {name}() {{
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
) -> Method {
    Method {
        definition: format!(
            r#"
    void {name}({field_type} value) {{
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
    {class_name}() {{
        this->self = {default_constructor_ext_fn_name}();
    }}"#,
        );

        let extern_fn = format!("    void* {default_constructor_ext_fn_name}();");

        Method {
            definition,
            extern_fn,
            include: String::new(),
        }
    } else {
        Method {
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
    virtual ~{class_name}() {{
        if (this->self != nullptr)
            {drop_ext_fn_name}(this->self);
    }}"#,
    );

    let extern_fn = format!("    void {drop_ext_fn_name}(void*);");

    Method {
        definition,
        extern_fn,
        include: String::new(),
    }
}
