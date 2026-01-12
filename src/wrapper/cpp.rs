use std::ops::Deref;

use super::*;
use crate::wrapper::base::*;

use class_definition::*;
use function_definition::*;
use quote::ToTokens;

pub mod class_definition;
pub mod function_definition;

impl ReusableWrapper {
    pub fn cpp(&self) -> String {
        match self {
            ReusableWrapper::Vec(inner) => gen_vec_wrapper_cpp(inner),
            ReusableWrapper::Result(inner) => gen_result_wrapper_cpp(inner),
        }
    }
}

fn gen_result_wrapper_cpp(inner: &WrapperType) -> String {
    let wrapper_name = format!("Rust{}Result", inner.name());
    let inner_name = inner.name();

    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{}_result", inner_name);
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{}_result", inner_name);
    let unwrap_err_ext_name = format!(
        "{EXPORTED_SYMBOLS_PREFIX}__unwrap_err_{}_result",
        inner_name
    );
    let is_err_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_err_{}_result", inner_name);

    let unwrap_return_type = match inner {
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Bool => "bool".to_string(),
        WrapperType::String => "std::string".to_string(),
        WrapperType::Struct(name) => name.to_string(),
        WrapperType::Enum(name) => name.to_string(),
        WrapperType::Vec(vec_inner_name) => match vec_inner_name.deref() {
            WrapperType::String => "std::vector<std::string>".to_string(),
            _ => format!("std::vector<{}>", vec_inner_name.name()),
        },
        WrapperType::Result(_) => {
            panic!("Nested Result types are not supported")
        }
    };

    let unwrap_ext_ret_type = match inner {
        WrapperType::String | WrapperType::Struct(_) | WrapperType::Vec(_) => "void*".to_string(),
        WrapperType::Bool
        | WrapperType::Enum(_)
        | WrapperType::FloatingPointNumber(_)
        | WrapperType::IntegerNumber(_) => inner_name.clone(),
        WrapperType::Result(_) => {
            panic!("Nested Result types are not supported")
        }
    };

    let ext_call = format!("{unwrap_ext_name}(this->self)");
    let unwrap_val_cast = match inner {
        WrapperType::String => format!("auto result = RustString({ext_call}).to_string();"),
        WrapperType::Struct(struct_name) => {
            format!("auto result = {struct_name}({ext_call});")
        }
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name = vec_inner.name();
            format!("auto result = Rust{vec_inner_name}Vec::from_raw({ext_call}).to_std();")
        }
        _ => format!("auto result = {ext_call};"),
    };

    let includes = match inner {
        WrapperType::Struct(struct_name) => format!(r#"#include "{struct_name}.h""#),
        WrapperType::Enum(enum_name) => format!(r#"#include "{enum_name}.h""#),
        WrapperType::Vec(vec_inner) => format!(r#"#include "vec_{}.h""#, vec_inner.name()),
        _ => String::new(),
    };

    format!(
        r#"
#ifndef {wrapper_name}__def
#define {wrapper_name}__def
{includes}

extern "C" {{
    void {drop_ext_name}(void* self);
    void* {unwrap_err_ext_name}(void* self);
    {unwrap_ext_ret_type} {unwrap_ext_name}(void * self);
    bool {is_err_ext_name}(void* self);
}}

class {wrapper_name} {{
    void* self = nullptr;
public:
    {wrapper_name}(void* self) : self(self) {{}}
    void* raw_ptr() {{
        return this->self;
    }}
    virtual ~{wrapper_name}() {{
        if (this->self != nullptr)
            {drop_ext_name}(this->self);
    }}

    bool is_err() {{
        return {is_err_ext_name}(this->self);
    }}

    {unwrap_return_type} unwrap() {{
        {unwrap_val_cast}
        return result;
    }}

    void* unwrap_err() {{
        return {unwrap_err_ext_name}(this->self);
    }}
}};
#endif
"#
    )
}

fn gen_vec_wrapper_cpp(inner: &WrapperType) -> String {
    let wrapper_name = format!("Rust{}Vec", inner.name());
    let inner_name = inner.name();
    let inner_cpp_name = match inner {
        WrapperType::String => "std::string".to_string(),
        _ => inner_name.clone(),
    };
    let push_ext_receiver = match inner {
        WrapperType::IntegerNumber(inner) | WrapperType::FloatingPointNumber(inner) => {
            format!("{inner} value")
        }
        WrapperType::Struct(inner) => format!("void* {inner}"),
        WrapperType::Vec(_) => unimplemented!("CPP: vec of vecs unimplemented!"),
        WrapperType::Bool => "bool value".to_string(),
        WrapperType::String => "const char* value".to_string(), // 2nd arg, len, is ignored for now
        WrapperType::Enum(_) => "int value".to_string(),
        WrapperType::Result(_) => {
            unimplemented!("CPP: vec of Result type unimplemented!")
        }
    };
    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{inner_name}_vec");
    let with_capacity_ext_name =
        format!("{EXPORTED_SYMBOLS_PREFIX}__with_capacity_{inner_name}_vec");
    let push_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__push_{inner_name}_vec");

    // New externs for reading
    let len_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__len_{inner_name}_vec");
    let get_ext_fn_name = format!("{EXPORTED_SYMBOLS_PREFIX}__get_{inner_name}_vec");

    let loop_expressions = match inner {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) | WrapperType::Bool => {
            format!("           {push_ext_fn_name}(rust_vec_ptr, elem);")
        }
        WrapperType::String => {
            format!("            {push_ext_fn_name}(rust_vec_ptr, elem.data());")
        }
        WrapperType::Struct(_) => {
            format!("            {push_ext_fn_name}(rust_vec_ptr, elem.self_ptr());")
        }
        WrapperType::Enum(_) => {
            format!("            {push_ext_fn_name}(rust_vec_ptr, static_cast<int>(elem));")
        }
        _ => panic!("Pushing to vec not supported"),
    };

    let includes = match inner {
        WrapperType::Struct(struct_name) => format!(r#"#include "{struct_name}.h""#),
        WrapperType::Enum(enum_name) => format!(r#"#include "{enum_name}.h""#),
        _ => String::new(),
    };

    // Element fetch expression per type
    let elem_fetch = match inner {
        WrapperType::IntegerNumber(_) | WrapperType::FloatingPointNumber(_) => {
            format!("auto elem = {get_ext_fn_name}(this->self, i);")
        }
        WrapperType::Bool => {
            format!("bool elem = {get_ext_fn_name}(this->self, i) != 0;")
        }
        WrapperType::String => {
            // get returns pointer to Rust String heap-allocated; convert to std::string and free via RustString wrapper
            format!(
                "auto elem_ptr = {get_ext_fn_name}(this->self, i); RustString rs(elem_ptr); auto elem = rs.to_string();"
            )
        }
        WrapperType::Struct(struct_name) => {
            format!(
                "auto elem_ptr = {get_ext_fn_name}(this->self, i); {struct_name} elem(elem_ptr);"
            )
        }
        WrapperType::Enum(_) => {
            format!("auto elem = {get_ext_fn_name}(this->self, i);")
        }
        WrapperType::Result(_) | WrapperType::Vec(_) => unreachable!(),
    };

    let get_ext_return_type = match inner {
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Bool => "u8".to_string(),
        WrapperType::String => "void*".to_string(),
        WrapperType::Struct(_) => "void*".to_string(),
        WrapperType::Enum(name) => name.to_string(),
        WrapperType::Vec(_) => unimplemented!("Vec of vecs not supported yet!"),
        WrapperType::Result(_) => unimplemented!("Vec of Result type not supported yet!"),
    };

    format!(
        r#"
#ifndef {wrapper_name}__def
#define {wrapper_name}__def

#include "base.h"
#include <vector>
{includes}

extern "C" {{
    void* {drop_ext_name}(void* self);
    void* {with_capacity_ext_name}(size_t capacity);
    void* {push_ext_fn_name}(void* self, {push_ext_receiver});
    usize {len_ext_fn_name}(void* self);
    {get_ext_return_type} {get_ext_fn_name}(void* self, usize index);
}}

class {wrapper_name} {{
    void* self = nullptr;
public:
    {wrapper_name}(void* self) : self(self) {{}}

    void* raw_ptr() {{
        return this->self;
    }}

    virtual ~{wrapper_name}() {{
        if (this->self != nullptr)
            {drop_ext_name}(this->self);
    }}

    static {wrapper_name} from_std(const std::vector<{inner_cpp_name}>& other) {{
        void* rust_vec_ptr = {with_capacity_ext_name}(other.size());
        for (const auto& elem : other) {{
{loop_expressions}
        }}
        return {wrapper_name}(rust_vec_ptr);
    }}

    static {wrapper_name} from_raw(void* raw) {{ return {wrapper_name}(raw); }}

    void leak() {{
        this->self = nullptr;
    }}

    std::vector<{inner_cpp_name}> to_std() {{
        std::vector<{inner_cpp_name}> out;
        auto len = {len_ext_fn_name}(this->self);
        out.reserve(len);
        for (usize i = 0; i < len; ++i) {{
            {elem_fetch}
            out.push_back(std::move(elem));
        }}
        return out;
    }}
}};

#endif
"#
    )
}

pub enum CppHeader {
    Class(ClassHeaderParts),
    Function(String),
}

pub struct CppFiles {
    pub header: CppHeader,
    pub source: Option<String>,
}

impl Wrapper {
    pub fn cpp(&self) -> CppFiles {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => CppFiles {
                header: CppHeader::Class(gen_class_definition_parts_from_struct(struct_wrapper)),
                source: None,
            },
            ParsedWrapper::Function(function_wrapper) => CppFiles {
                header: CppHeader::Function(gen_function_declaration(function_wrapper)),
                source: Some(gen_function_definition(function_wrapper)),
            },
            ParsedWrapper::ImplBlock(impl_block_wrapper) => CppFiles {
                header: CppHeader::Class(gen_class_definition_parts_from_impl_block(
                    impl_block_wrapper,
                )),
                source: None,
            },
            ParsedWrapper::Enum(enum_wrapper) => CppFiles {
                header: CppHeader::Function(gen_enum_declaration(enum_wrapper)),
                source: None,
            },
        }
    }
}

struct Methods {
    getter: Option<Method>,
    setter: Option<Method>,
}

struct Method {
    definition: String,
    extern_fn: String,
    include: String,
}

struct ReturnTypes {
    ext_return_type: String,
    return_type: String,
    return_cast: String,
    return_type_includes: String,
}

fn map_return_type(return_wrapper: &Option<FunctionReturnWrapper>) -> ReturnTypes {
    match return_wrapper {
        Some(FunctionReturnWrapper {
            wrapper_type:
                WrapperType::IntegerNumber(_) | WrapperType::Bool | WrapperType::FloatingPointNumber(_),
            return_type,
        }) => ReturnTypes {
            ext_return_type: return_type.to_token_stream().to_string(),
            return_type: return_type.to_token_stream().to_string(),
            return_cast: "return result;".to_string(),
            return_type_includes: String::new(),
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::String,
            ..
        }) => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "std::string".to_string(),
            return_cast: "
auto rust_str = RustString(result);
return rust_str.to_string();"
                .to_string(),
            return_type_includes: String::new(),
        },
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Struct(_),
            return_type,
        }) => {
            let struct_type = return_type.to_token_stream().to_string();
            ReturnTypes {
                ext_return_type: "void*".to_string(),
                return_type: struct_type.clone(),
                return_cast: format!(
                    "
return {}(result);",
                    struct_type
                ),
                return_type_includes: format!("#include \"{struct_type}.h\""),
            }
        }
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Vec(inner),
            ..
        }) => {
            // Determine C++ inner type
            let inner_cpp = match inner.as_ref() {
                WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.clone(),
                WrapperType::Bool => "bool".to_string(),
                WrapperType::String => "std::string".to_string(),
                WrapperType::Struct(name) => name.clone(),
                WrapperType::Enum(name) => name.clone(),
                WrapperType::Vec(_) => unimplemented!("Nested vectors not supported"),
                WrapperType::Result(_) => {
                    unimplemented!("Vec of Result type not supported yet!")
                }
            };
            let rust_vec_name = format!("Rust{}Vec", inner.name());
            let return_cast = format!(
                r#"
auto rust_vec = {rust_vec_name}::from_raw(result);
return rust_vec.to_std();
"#
            );
            let mut includes = String::from("#include <vector>\n");
            includes.push_str(&format!("#include \"vec_{}.h\"", inner.name()));
            match inner.as_ref() {
                WrapperType::Struct(inner_struct) => {
                    // For struct vector, ensure struct header is included
                    includes.push_str(&format!("\n#include \"{inner_struct}.h\""));
                }
                WrapperType::Enum(inner_enum) => {
                    // For enum vector, ensure enum header is included
                    includes.push_str(&format!("\n#include \"{inner_enum}.h\""));
                }
                _ => {}
            }
            ReturnTypes {
                ext_return_type: "void*".to_string(),
                return_type: format!("std::vector<{inner_cpp}>").to_string(),
                return_cast,
                return_type_includes: includes,
            }
        }
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Enum(_),
            return_type,
        }) => {
            let enum_type = return_type.to_token_stream().to_string();
            ReturnTypes {
                ext_return_type: enum_type.clone(),
                return_type: enum_type.clone(),
                return_cast: "return result;".to_string(),
                return_type_includes: format!("#include \"{}.h\"", enum_type),
            }
        }
        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Result(inner_ok_type),
            ..
        }) => {
            let inner_name = inner_ok_type.name();
            let inner_type_include = match inner_ok_type.as_ref() {
                WrapperType::Struct(struct_name) => format!("#include \"{}.h\"\n", struct_name),
                WrapperType::Enum(enum_name) => format!("#include \"{}.h\"\n", enum_name),
                _ => String::new(),
            };
            ReturnTypes {
                ext_return_type: "void*".to_string(),
                return_type: match inner_ok_type.as_ref() {
                    WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => {
                        t.clone()
                    }
                    WrapperType::Bool => "bool".to_string(),
                    WrapperType::String => "std::string".to_string(),
                    WrapperType::Struct(name) => name.clone(),
                    WrapperType::Enum(name) => name.clone(),
                    WrapperType::Vec(vec_inner_name) => match vec_inner_name.deref() {
                        WrapperType::String => "std::vector<std::string>".to_string(),
                        _ => format!("std::vector<{}>", vec_inner_name.name()),
                    },
                    WrapperType::Result(_) => {
                        panic!("Nested Result types are not supported")
                    }
                },
                return_cast: format!(
                    r#"auto rust_result = Rust{inner_name}Result(result);
return rust_result.is_err() ? throw RustException(rust_result.unwrap_err()) : rust_result.unwrap();"#
                ),

                return_type_includes: format!(
                    "{inner_type_include}#include \"result_{inner_name}.h\""
                ),
            }
        }
        None => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "void".to_string(),
            return_cast: "".to_string(),
            return_type_includes: String::new(),
        },
    }
}

fn gen_enum_declaration(enum_wrapper: &EnumWrapper) -> String {
    let enum_name = &enum_wrapper.name;

    let variants: Vec<String> = enum_wrapper
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.name;
            match &variant.discriminant {
                Some(discriminant) => {
                    format!("    {} = {}", variant_name, discriminant)
                }
                None => {
                    format!("    {}", variant_name)
                }
            }
        })
        .collect();

    format!(
        r#"
#ifndef {enum_name}_h
#define {enum_name}_h

#include "base.h"

enum class {enum_name} {{
{}
}};

#endif
"#,
        variants.join(",\n")
    )
}
