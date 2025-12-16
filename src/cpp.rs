use crate::wrapper::base::*;
use crate::wrapper::cpp::{CppFiles, CppHeader};
use crate::{CPP_CODE_DIR, GEN_CODE_DIR, ReusableWrapper, Wrapper, create_file, insert_after};
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
        create_file(source, source_full_path);
    }

    for reusable_wrapper in &wrapper.reusable_wrappers {
        let file_name = match reusable_wrapper {
            ReusableWrapper::Vec(inner) => format!("vec_{}.h", inner.name()),
        };
        let source_full_path = cpp_path.join(file_name);
        if CPP_REUSABLE_WRAPPER_GENERATED
            .lock()
            .expect("Mutex lock failed during locking for reusable wrapper")
            .insert(source_full_path.clone())
        {
            create_file(reusable_wrapper.cpp(), source_full_path);
        }
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

            insert_after(
                crate::wrapper::cpp::class_definition::INCLUDES_MARKER,
                class_header_parts.includes,
                &path,
            );
            insert_after(
                crate::wrapper::cpp::class_definition::EXTERN_FNS_MARKER,
                class_header_parts.extern_fns,
                &path,
            );
            insert_after(
                crate::wrapper::cpp::class_definition::METHOD_DEFINITIONS_MARKER,
                class_header_parts.method_definitions,
                &path,
            );
        }
        CppHeader::Function(function_header) => create_file(function_header, path),
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
}}

class RustString {{
    void* self;
public:
    RustString(void* self) : self(self) {{}}
    ~RustString() {{
        {RUST_STRING_DROP_FN_NAME}(self);
    }}
    std::string to_string() {{
        auto ptr = {RUST_STRING_DATA_FN_NAME}(self);
        auto len = {RUST_STRING_LEN_FN_NAME}(self);
        return std::string(ptr, len);
    }}
}};

#endif

"#
    )
}
