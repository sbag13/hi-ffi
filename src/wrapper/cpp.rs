use std::fmt::Display;
use std::ops::Deref;

use super::*;
use crate::wrapper::base::*;

use class_definition::*;
use function_definition::*;
use quote::ToTokens;

pub mod class_definition;
pub mod function_definition;

impl ReusableWrapper {
    pub fn cpp(&self) -> CppFiles {
        match self {
            ReusableWrapper::Vec(inner) => gen_vec_wrapper_cpp(inner),
            ReusableWrapper::Result(inner) => gen_result_wrapper_cpp(inner),
            ReusableWrapper::Option(inner) => gen_option_wrapper_cpp(inner),
        }
    }
}

// Common helper functions
fn gen_common_destructor(wrapper_name: &str, drop_ext_name: &str) -> String {
    format!(
        "{wrapper_name}::~{wrapper_name}() {{\n    if (this->self != nullptr)\n        {drop_ext_name}(this->self);\n}}"
    )
}

fn gen_common_constructor(wrapper_name: &str) -> String {
    format!("{wrapper_name}::{wrapper_name}(void* self) : self(self) {{}}")
}

fn gen_common_raw_ptr(wrapper_name: &str) -> String {
    format!("void* {wrapper_name}::raw_ptr() {{\n    return this->self;\n}}")
}

fn gen_option_wrapper_cpp(inner: &WrapperType) -> CppFiles {
    let wrapper_name = format!("Rust{}Option", inner.name());
    let inner_name = inner.name();

    let includes = inner_include(inner).unwrap_or_default();
    let forward_class_declaration = forward_class_declaration(inner);

    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{}_option", inner_name);
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{}_option", inner_name);
    let is_some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_some_{}_option", inner_name);

    let inner_ext_ret_type = ext_type(inner);
    let inner_cpp_type = cpp_type(inner);

    let ext_call = format!("{unwrap_ext_name}(this->self)");
    let unwrap_val_cast = inner_value_cast(inner, ext_call);

    let unwrap_return_expr = match inner {
        WrapperType::UnitExpr => "return;",
        _ => "return result;",
    };

    let none_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__none_{}_option", inner_name);
    let some_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__some_{}_option", inner_name);
    let inner_ext_arg_type = match inner {
        WrapperType::String => "const char*",
        _ => &ext_type(inner),
    };

    let has_value_cast = match inner {
        WrapperType::String => "other.value().data()",
        WrapperType::Struct(_) => "other.value().self_ptr()",
        _ => "other.value()",
    };

    let header = format!(
        r#"
#ifndef {wrapper_name}__def
#define {wrapper_name}__def
#include "base.h"
{includes}{forward_class_declaration}

extern "C" {{
    void {drop_ext_name}(void* self);
    {inner_ext_ret_type} {unwrap_ext_name}(void * self);
    bool {is_some_ext_name}(void* self);
    void* {some_ext_name}({inner_ext_arg_type} val);
    void* {none_ext_name}();
}}

class {wrapper_name} {{
    void* self = nullptr;
public:
    {wrapper_name}(void* self);
    void* raw_ptr();
    virtual ~{wrapper_name}();
    bool is_some();
    {inner_cpp_type} unwrap();
    static {wrapper_name} from_std(const std::optional<{inner_cpp_type}>& other);
    void* leak();
    std::optional<{inner_cpp_type}> to_std();
}};
#endif
"#
    );

    let constructor = gen_common_constructor(&wrapper_name);
    let raw_ptr = gen_common_raw_ptr(&wrapper_name);
    let destructor = gen_common_destructor(&wrapper_name, &drop_ext_name);

    let source = format!(
        r#"
#include "option_{inner_name}.h"

{constructor}

{raw_ptr}

{destructor}

bool {wrapper_name}::is_some() {{
    return {is_some_ext_name}(this->self);
}}

{inner_cpp_type} {wrapper_name}::unwrap() {{
    {unwrap_val_cast}
    {unwrap_return_expr}
}}

{wrapper_name} {wrapper_name}::from_std(const std::optional<{inner_cpp_type}>& other) {{
    if (other.has_value()) {{
        auto other_value = {has_value_cast};
        return {wrapper_name}({some_ext_name}(other_value));
    }} else {{
        return {none_ext_name}();
    }}
}}

void* {wrapper_name}::leak() {{
    void* result = this->self;
    this->self = nullptr;
    return result;
}}

std::optional<{inner_cpp_type}> {wrapper_name}::to_std() {{
    if (this->is_some()) {{
        return std::optional<{inner_cpp_type}>(this->unwrap());
    }} else {{
        return std::optional<{inner_cpp_type}>();
    }}
}}
"#
    );

    CppFiles {
        header: CppHeader::Reusable(header),
        source: Some(CppSource::Reusable(source)),
    }
}

fn gen_result_wrapper_cpp(inner: &WrapperType) -> CppFiles {
    let wrapper_name = format!("Rust{}Result", inner.name());
    let inner_name = inner.name();

    let drop_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__drop_{}_result", inner_name);
    let unwrap_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__unwrap_{}_result", inner_name);
    let unwrap_err_ext_name = format!(
        "{EXPORTED_SYMBOLS_PREFIX}__unwrap_err_{}_result",
        inner_name
    );
    let is_err_ext_name = format!("{EXPORTED_SYMBOLS_PREFIX}__is_err_{}_result", inner_name);
    let unwrap_return_type = cpp_type(inner);
    let unwrap_ext_ret_type = ext_type(inner);

    let ext_call = format!("{unwrap_ext_name}(this->self)");
    let unwrap_val_cast = inner_value_cast(inner, ext_call);

    let includes = inner_include(inner).unwrap_or_default();

    let forward_class_declaration = forward_class_declaration(inner);

    let unwrap_return_expr = match inner {
        WrapperType::UnitExpr => "return;",
        _ => "return result;",
    };

    let header = format!(
        r#"
#ifndef {wrapper_name}__def
#define {wrapper_name}__def
#include "base.h"
{includes}{forward_class_declaration}

extern "C" {{
    void {drop_ext_name}(void* self);
    void* {unwrap_err_ext_name}(void* self);
    {unwrap_ext_ret_type} {unwrap_ext_name}(void * self);
    bool {is_err_ext_name}(void* self);
}}

class {wrapper_name} {{
    void* self = nullptr;
public:
    {wrapper_name}(void* self);
    void* raw_ptr();
    virtual ~{wrapper_name}();
    bool is_err();
    {unwrap_return_type} unwrap();
    void* unwrap_err();
}};
#endif
"#
    );

    let constructor = gen_common_constructor(&wrapper_name);
    let raw_ptr = gen_common_raw_ptr(&wrapper_name);
    let destructor = gen_common_destructor(&wrapper_name, &drop_ext_name);

    let source = format!(
        r#"
#include "result_{inner_name}.h"

{constructor}

{raw_ptr}

{destructor}

bool {wrapper_name}::is_err() {{
    return {is_err_ext_name}(this->self);
}}

{unwrap_return_type} {wrapper_name}::unwrap() {{
    {unwrap_val_cast}
    {unwrap_return_expr}
}}

void* {wrapper_name}::unwrap_err() {{
    return {unwrap_err_ext_name}(this->self);
}}"#
    );

    CppFiles {
        header: CppHeader::Reusable(header),
        source: Some(CppSource::Reusable(source)),
    }
}

fn gen_vec_wrapper_cpp(inner: &WrapperType) -> CppFiles {
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
        WrapperType::Option(_) => todo!("option in vec push cpp"),
        WrapperType::UnitExpr => {
            panic!("Pushing () to vec not supported")
        }
        WrapperType::Trait(_) => panic!("Trait objects in cpp vec not supported"),
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

    let includes = inner_include(inner).unwrap_or_default();

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
        WrapperType::Option(_) => todo!("option in vec cpp"),
        WrapperType::Result(_) | WrapperType::Vec(_) => unreachable!(),
        WrapperType::UnitExpr => unreachable!(),
        WrapperType::Trait(_) => panic!("Trait objects in cpp vec not supported"),
    };

    let get_ext_return_type = ext_type(inner);
    let forward_class_declaration = forward_class_declaration(inner);

    let header = format!(
        r#"
#ifndef {wrapper_name}__def
#define {wrapper_name}__def

#include "base.h"
#include <vector>
{includes}{forward_class_declaration}

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
    {wrapper_name}(void* self);

    void* raw_ptr();

    virtual ~{wrapper_name}();

    static {wrapper_name} from_std(const std::vector<{inner_cpp_name}>& other);

    void* leak();

    std::vector<{inner_cpp_name}> to_std();
}};

#endif
"#
    );

    let constructor = gen_common_constructor(&wrapper_name);
    let raw_ptr = gen_common_raw_ptr(&wrapper_name);
    let destructor = gen_common_destructor(&wrapper_name, &drop_ext_name);

    let source = format!(
        r#"
#include "vec_{inner_name}.h"

{constructor}

{raw_ptr}

{destructor}

{wrapper_name} {wrapper_name}::from_std(const std::vector<{inner_cpp_name}>& other) {{
    void* rust_vec_ptr = {with_capacity_ext_name}(other.size());
    for (const auto& elem : other) {{
{loop_expressions}
    }}
    return {wrapper_name}(rust_vec_ptr);
}}

void* {wrapper_name}::leak() {{
    void* result = this->self;
    this->self = nullptr;
    return result;
}}

std::vector<{inner_cpp_name}> {wrapper_name}::to_std() {{
    std::vector<{inner_cpp_name}> out;
    auto len = {len_ext_fn_name}(this->self);
    out.reserve(len);
    for (usize i = 0; i < len; ++i) {{
        {elem_fetch}
        out.push_back(std::move(elem));
    }}
    return out;
}}
"#
    );

    CppFiles {
        header: CppHeader::Reusable(header),
        source: Some(CppSource::Reusable(source)),
    }
}

fn inner_include(inner: &WrapperType) -> Option<String> {
    match inner {
        WrapperType::Struct(name) | WrapperType::Enum(name) => {
            Some(format!(r#"#include "{name}.h""#))
        }
        WrapperType::Vec(vec_inner) => Some(format!(r#"#include "vec_{}.h""#, vec_inner.name())),
        WrapperType::Option(opt_inner) => {
            Some(format!(r#"#include "option_{}.h""#, opt_inner.name()))
        }
        WrapperType::Result(result_inner) => {
            Some(format!(r#"#include "result_{}.h""#, result_inner.name()))
        }
        WrapperType::String => Some("#include <string>".to_string()),
        _ => None,
    }
}

fn forward_class_declaration(inner: &WrapperType) -> String {
    match inner {
        WrapperType::Enum(enum_name) => format!("\nenum class {enum_name};"),
        WrapperType::Struct(struct_name) => format!("\nclass {struct_name};"),
        _ => String::new(),
    }
}

fn ext_type(inner: &WrapperType) -> String {
    match inner {
        WrapperType::String
        | WrapperType::Struct(_)
        | WrapperType::Vec(_)
        | WrapperType::Option(_)
        | WrapperType::Result(_) => "void*".to_string(),
        WrapperType::Bool
        | WrapperType::Enum(_)
        | WrapperType::FloatingPointNumber(_)
        | WrapperType::IntegerNumber(_) => inner.name().to_string(),
        WrapperType::UnitExpr => "void".to_string(),
        WrapperType::Trait(_) => panic!("[ext_type]: Trait objects in cpp not supported"),
    }
}

fn inner_value_cast(inner: &WrapperType, ext_call: impl Display) -> String {
    match inner {
        WrapperType::String => format!("auto result = RustString({ext_call}).to_string();"),
        WrapperType::Struct(struct_name) => {
            format!("auto result = {struct_name}({ext_call});")
        }
        WrapperType::Vec(vec_inner) => {
            let vec_inner_name = vec_inner.name();
            format!("auto result = Rust{vec_inner_name}Vec({ext_call}).to_std();")
        }
        WrapperType::Option(opt_inner) => {
            let opt_inner_name = opt_inner.name();
            format!("auto result = Rust{opt_inner_name}Option({ext_call}).to_std();")
        }
        WrapperType::Result(result_inner) => {
            let result_inner_name = result_inner.name();
            format!("auto result = Rust{result_inner_name}Result({ext_call}).to_std();")
        }
        WrapperType::UnitExpr => "".to_string(),
        _ => format!("auto result = {ext_call};"),
    }
}

pub enum CppHeader {
    Class(ClassHeaderParts),
    Function(String),
    Reusable(String),
}

pub enum CppSource {
    Class(ClassSourceParts),
    Function(String),
    Reusable(String),
}

pub struct CppFiles {
    pub header: CppHeader,
    pub source: Option<CppSource>,
}

impl Wrapper {
    pub fn cpp(&self) -> CppFiles {
        match &self.parsed {
            ParsedWrapper::Struct(struct_wrapper) => CppFiles {
                header: CppHeader::Class(gen_class_definition_parts_from_struct(struct_wrapper)),
                source: Some(CppSource::Class(gen_methods_definitions_from_struct(
                    struct_wrapper,
                ))),
            },
            ParsedWrapper::Function(function_wrapper) => CppFiles {
                header: CppHeader::Function(gen_function_declaration(function_wrapper)),
                source: Some(CppSource::Function(gen_function_definition(
                    function_wrapper,
                ))),
            },
            ParsedWrapper::ImplBlock(impl_block_wrapper) => CppFiles {
                header: CppHeader::Class(gen_class_definition_parts_from_impl_block(
                    impl_block_wrapper,
                )),
                source: Some(CppSource::Class(gen_class_source_from_impl_block(
                    impl_block_wrapper,
                ))),
            },
            ParsedWrapper::Enum(enum_wrapper) => CppFiles {
                header: CppHeader::Function(gen_enum_declaration(enum_wrapper)),
                source: None,
            },
            ParsedWrapper::Trait(trait_wrapper) => CppFiles {
                header: CppHeader::Class(gen_interface_class(trait_wrapper)),
                source: Some(CppSource::Class(gen_trait_methods_definitions(
                    trait_wrapper,
                ))),
            },
        }
    }
}

struct Methods {
    getter: Option<Method>,
    setter: Option<Method>,
}

struct Method {
    declaration: String,
    definition: String,
    extern_fn: String,
    include: String,
}

struct ReturnTypes {
    ext_return_type: String,
    return_type: String,
    return_cast: String,
    return_type_includes: HashSet<String>,
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
            return_type_includes: HashSet::new(),
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
            return_type_includes: HashSet::new(),
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
                return_type_includes: HashSet::from([format!("#include \"{struct_type}.h\"")]),
            }
        }

        Some(FunctionReturnWrapper {
            wrapper_type: wt @ WrapperType::Vec(inner),
            ..
        }) => {
            let rust_vec_name = format!("Rust{}Vec", inner.name());
            let return_cast = format!(
                r#"
auto rust_vec = {rust_vec_name}(result);
return rust_vec.to_std();
"#
            );
            let mut includes = HashSet::from([
                String::from("#include <vector>\n"),
                format!("#include \"vec_{}.h\"", inner.name()),
            ]);
            if let Some(inner_include) = inner_include(inner) {
                includes.insert(inner_include);
            }
            ReturnTypes {
                ext_return_type: "void*".to_string(),
                return_type: cpp_type(wt),
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
                return_type_includes: HashSet::from([format!("#include \"{}.h\"", enum_type)]),
            }
        }

        Some(FunctionReturnWrapper {
            wrapper_type: wt @ WrapperType::Option(inner),
            ..
        }) => {
            let inner_name = inner.name();
            let mut includes = HashSet::from([
                String::from("#include <optional>\n"),
                format!("#include \"option_{inner_name}.h\""),
            ]);
            if let Some(inner_include) = inner_include(inner) {
                includes.insert(inner_include);
            }
            ReturnTypes {
                ext_return_type: "void*".to_string(),
                return_type: cpp_type(wt),
                return_cast: format!(
                    r#"auto rust_opt = Rust{inner_name}Option(result);
return rust_opt.to_std();"#
                ),
                return_type_includes: includes,
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
                return_type: cpp_type(inner_ok_type.deref()),
                return_cast: format!(
                    r#"auto rust_result = Rust{inner_name}Result(result);
return rust_result.is_err() ? throw RustException(rust_result.unwrap_err()) : rust_result.unwrap();"#
                ),

                return_type_includes: HashSet::from([format!(
                    "{inner_type_include}#include \"result_{inner_name}.h\""
                )]),
            }
        }

        None
        | Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::UnitExpr,
            ..
        }) => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: "void".to_string(),
            return_cast: "".to_string(),
            return_type_includes: HashSet::new(),
        },

        Some(FunctionReturnWrapper {
            wrapper_type: WrapperType::Trait(trait_name),
            ..
        }) => ReturnTypes {
            ext_return_type: "void*".to_string(),
            return_type: format!("std::shared_ptr<{trait_name}>"),
            return_cast: format!(
                r#"
return std::make_shared<{trait_name}RustImpl>({trait_name}RustImpl(result));"#
            ),
            return_type_includes: HashSet::from([
                format!("#include \"{trait_name}.h\""),
                "#include <memory>".to_string(),
            ]),
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

pub(crate) fn cpp_type(wrapper: &WrapperType) -> String {
    match wrapper {
        WrapperType::String => "std::string".to_string(),
        WrapperType::Vec(inner) => format!("std::vector<{}>", cpp_type(inner)),
        WrapperType::Option(inner) => format!("std::optional<{}>", cpp_type(inner)),
        WrapperType::Bool => "bool".to_string(),
        WrapperType::IntegerNumber(t) | WrapperType::FloatingPointNumber(t) => t.to_string(),
        WrapperType::Struct(struct_name) => struct_name.to_string(),
        WrapperType::Enum(enum_name) => enum_name.to_string(),
        WrapperType::Result(_inner) => {
            panic!("'cpp_type' shouldn't be called for a Result type - this is a bug; {wrapper:?}")
        }
        WrapperType::UnitExpr => "void".to_string(),
        WrapperType::Trait(_) => panic!("[cpp_type]: Trait objects in cpp type not supported"),
    }
}
