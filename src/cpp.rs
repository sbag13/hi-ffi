use crate::wrapper::base::*;
use crate::wrapper::cpp::class_definition::ClassSourceParts;
use crate::wrapper::cpp::{CppFiles, CppHeader, CppSource};
use crate::{
    CPP_CODE_DIR, GEN_CODE_DIR, ReusableWrapper, Wrapper, append_to_file, create_file,
    insert_after, insert_after_if_not_present,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

static CPP_REUSABLE_WRAPPER_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub(crate) fn write_cpp_code(wrapper: &Wrapper) {
    let cpp_path = Path::new(GEN_CODE_DIR).join(CPP_CODE_DIR);
    std::fs::create_dir_all(&cpp_path).expect("Unable to create cpp directory");

    let code_base_path = cpp_path.join("base.h");
    // if !code_base_path.exists() { // TODO: uncomment when stable implementation is ready
    create_file(cpp_code_base(), code_base_path);
    // }

    let header_file_name = format!("{}.h", wrapper.name());
    let source_file_name = format!("{}.cpp", wrapper.name());

    let header_full_path = cpp_path.join(header_file_name);
    let source_full_path = cpp_path.join(source_file_name);

    let CppFiles { header, source } = wrapper.cpp();
    write_header(header, header_full_path);

    if let Some(source) = source {
        write_source(source, source_full_path);
    }

    for reusable_wrapper in &wrapper.reusable_wrappers {
        let (header_file_name, source_file_name) = match reusable_wrapper {
            ReusableWrapper::Vec(inner) => (
                format!("vec_{}.h", inner.name()),
                format!("vec_{}.cpp", inner.name()),
            ),
            ReusableWrapper::Result(inner) => (
                format!("result_{}.h", inner.name()),
                format!("result_{}.cpp", inner.name()),
            ),
            ReusableWrapper::Option(inner) => (
                format!("option_{}.h", inner.name()),
                format!("option_{}.cpp", inner.name()),
            ),
        };
        let header_full_path = cpp_path.join(header_file_name);
        let source_full_path = cpp_path.join(source_file_name);
        if CPP_REUSABLE_WRAPPER_GENERATED
            .lock()
            .expect("Mutex lock failed during locking for reusable wrapper")
            .insert(source_full_path.clone())
        {
            let CppFiles { header, source } = reusable_wrapper.cpp();
            let CppHeader::Reusable(header) = header else {
                panic!("Unexpected cpp header type for reusable wrapper");
            };
            let Some(CppSource::Reusable(source)) = source else {
                panic!("Unexpected cpp source type for reusable wrapper");
            };
            create_file(header, header_full_path);
            create_file(source, source_full_path);
        }
    }
}

fn write_source(source: CppSource, path: impl AsRef<Path>) {
    match source {
        CppSource::Function(fn_source) => {
            create_file(fn_source, path);
        }
        CppSource::Class(ClassSourceParts {
            base,
            methods_definitions,
        }) => {
            let mut locked_set = CPP_CLASS_GENERATED.lock().expect("Mutex lock failed");
            if locked_set.insert(path.as_ref().into()) {
                create_file(base, path.as_ref());
            }
            append_to_file(methods_definitions, path);
        }
        _ => (),
    }
}

static CPP_CLASS_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn write_header(header: CppHeader, path: impl AsRef<Path>) {
    match header {
        CppHeader::Class(class_header_parts) => {
            let mut locked_set = CPP_CLASS_GENERATED.lock().expect("Mutex lock failed");
            if locked_set.insert(path.as_ref().into()) {
                create_file(class_header_parts.class_definition, path.as_ref());
            }

            for include in &class_header_parts.includes {
                insert_after_if_not_present(
                    crate::wrapper::cpp::class_definition::INCLUDES_MARKER,
                    include,
                    &path,
                );
            }
            insert_after(
                crate::wrapper::cpp::class_definition::EXTERN_FNS_MARKER,
                class_header_parts.extern_fns,
                &path,
            );
            insert_after(
                crate::wrapper::cpp::class_definition::METHOD_DEFINITIONS_MARKER,
                class_header_parts.method_declarations,
                &path,
            );
        }
        CppHeader::Function(function_header) => create_file(function_header, path),
        _ => (),
    }
}

fn cpp_code_base() -> String {
    format!(
        r#"
#ifndef BASE_H
#define BASE_H

#include <cstdint>
#include <cstddef>
#include <string>
#include <optional>

using u8 = uint8_t;
using u16 = uint16_t;
using u32 = uint32_t;
using u64 = uint64_t;

using i8 = int8_t;
using i16 = int16_t;
using i32 = int32_t;
using i64 = int64_t;

using f32 = float;
using f64 = double;

using usize = size_t;

extern "C" {{
    void {SLICE_DROP_FN_NAME}(void*);
    char* {SLICE_GET_PTR_FN_NAME}(void*);
    usize {SLICE_GET_LEN_FN_NAME}(void*);

    void {RUST_STRING_DROP_FN_NAME}(void*);
    char* {RUST_STRING_DATA_FN_NAME}(void*);
    usize {RUST_STRING_LEN_FN_NAME}(void*);

    void {RUST_ARC_DYN_ERR_DROP_FN_NAME}(void*);
    void* {RUST_ARC_DYN_ERR_DESC_FN_NAME}(void*);
    void* {RUST_ARC_DYN_ERR_SOURCE_FN_NAME}(void*);
    void* {RUST_ARC_DYN_ERR_CLONE_FN_NAME}(void*);

    void {RUST_REF_DYN_ERR_DROP_FN_NAME}(void*);
    void* {RUST_REF_DYN_ERR_DESC_FN_NAME}(void*);
    void* {RUST_REF_DYN_ERR_SOURCE_FN_NAME}(void*);
}}

class RustString {{
    void* self;
public:
    RustString(void* self) : self(self) {{}};
    ~RustString() {{
        {RUST_STRING_DROP_FN_NAME}(self);
    }}
    std::string to_string() {{
        auto ptr = {RUST_STRING_DATA_FN_NAME}(self);
        auto len = {RUST_STRING_LEN_FN_NAME}(self);
        return std::string(ptr, len);
    }}
}};

class RustException : public std::exception {{
    void* rust_error;
    void* source_error;
    std::string description;

public:
    // Move constructor
    RustException(RustException&& other) noexcept
        : rust_error(other.rust_error),
        source_error(other.source_error),
        description(std::move(other.description)) {{
        other.rust_error = nullptr;
        other.source_error = nullptr;
    }}

    RustException(void* rust_error) : rust_error(rust_error), source_error(nullptr) {{
        auto rust_desc_string = RustString({RUST_ARC_DYN_ERR_DESC_FN_NAME}(rust_error));
        this->description = rust_desc_string.to_string();
    }}
    RustException(void* rust_error, void* source_error) : rust_error(rust_error), source_error(source_error) {{
        auto rust_desc_string = RustString({RUST_REF_DYN_ERR_DESC_FN_NAME}(source_error));
        this->description = rust_desc_string.to_string();
    }}
    ~RustException() {{
        if (rust_error != nullptr) {{
            {RUST_ARC_DYN_ERR_DROP_FN_NAME}(rust_error);
        }}
        if (source_error != nullptr) {{
            {RUST_REF_DYN_ERR_DROP_FN_NAME}(source_error);
        }}
    }}
    char const* what() const noexcept override {{
        return this->description.c_str();
    }}

    std::optional<RustException> source() const {{

        void* source_dyn_err_ptr = nullptr;
        if (source_error == nullptr) {{
            source_dyn_err_ptr = {RUST_ARC_DYN_ERR_SOURCE_FN_NAME}(rust_error);
        }} else {{
            source_dyn_err_ptr = {RUST_REF_DYN_ERR_SOURCE_FN_NAME}(source_error);
        }}
        
        if (source_dyn_err_ptr == nullptr) {{
            return std::nullopt;
        }}
        
        auto err_clone_ptr = {RUST_ARC_DYN_ERR_CLONE_FN_NAME}(rust_error);
        return std::optional(RustException(err_clone_ptr, source_dyn_err_ptr));
    }}
}};

#endif

"#
    )
}
