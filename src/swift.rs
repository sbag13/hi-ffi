use crate::wrapper::swift::class_definition::gen_empty_class_definition;
use crate::wrapper::swift::{SwiftCode, gen_swift_result_declarations, gen_swift_vec_declarations, gen_swift_option_declarations};
use crate::wrapper::{ParsedWrapper, base::*};
use crate::{
    GEN_CODE_DIR, ReusableWrapper, Wrapper, append_to_file, create_file, insert_after,
};
use std::collections::HashSet;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, Once};

const SWIFT_CODE_DIR: &str = "swift/";

static SWIFT_C_HEADER_RECREATED: Once = Once::new();

static SWIFT_CLASS_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub(crate) fn write_swift_code(wrapper: &Wrapper) {
    let swift_path = Path::new(GEN_CODE_DIR).join(SWIFT_CODE_DIR);

    let c_ffi_package_path = swift_path.join("CFfiModule");
    let c_ffi_module_path = c_ffi_package_path.join("Sources/CFfiModule");
    let ffi_package_path = swift_path.join("FfiModule");
    let ffi_module_path = ffi_package_path.join("Sources/FfiModule");

    let swift_header_path = c_ffi_module_path.join("ffi_swift.h");

    SWIFT_C_HEADER_RECREATED.call_once(|| {
        // recreate swift packages
        let _ = std::fs::remove_dir_all(&swift_path);

        std::fs::create_dir_all(&c_ffi_module_path).expect("Unable to create CFfi directory");
        std::fs::create_dir_all(&ffi_module_path).expect("Unable to create Ffi directory");

        let c_ffi_package_file = c_ffi_package_path.join("Package.swift");
        create_file(swift_c_ffi_package_definition(), c_ffi_package_file);

        let ffi_package_file = ffi_package_path.join("Package.swift");
        create_file(swift_ffi_package_definition(), ffi_package_file);

        let module_map = c_ffi_module_path.join("module.modulemap");
        let package_name = std::env::var("CARGO_PKG_NAME").expect("Package name expected");
        create_file(clang_module_map(package_name), module_map);

        create_file(swift_c_header_code_base(), &swift_header_path);

        let swift_code_base_path = ffi_module_path.join("base.swift");
        create_file(swift_code_base(), swift_code_base_path);
    });

    let swift_code = wrapper.swift();

    match wrapper.parsed {
        ParsedWrapper::Enum(_) => insert_after(ENUM_DEFINITIONS_MARKER, swift_code.header(), &swift_header_path),
        _ => append_to_file(swift_code.header(), &swift_header_path),
    };

    // Add vector function declarations to header
    for reusable_wrapper in &wrapper.reusable_wrappers {
        let declarations = match reusable_wrapper {
            ReusableWrapper::Vec(inner) => gen_swift_vec_declarations(inner),
            ReusableWrapper::Result(inner) => gen_swift_result_declarations(inner),
            ReusableWrapper::Option(inner) => gen_swift_option_declarations(inner),
        };
        append_to_file(declarations, &swift_header_path);
    }

    let source_file_name = format!("{}.swift", wrapper.name());
    let source_path = ffi_module_path.join(source_file_name);

    match swift_code {
        SwiftCode::Class { source, .. } => {
            let mut locked_set = SWIFT_CLASS_GENERATED.lock().expect("Mutex lock failed");
            if locked_set.insert(source_path.clone()) {
                create_file(gen_empty_class_definition(wrapper.name()), &source_path);
            }

            insert_after(
                crate::wrapper::swift::class_definition::METHOD_DEFINITIONS_MARKER,
                source,
                &source_path,
            );
        }
        SwiftCode::Function { source, .. } | SwiftCode::Enum { source, .. } => {
            create_file(source, &source_path)
        }
    }

    // Generate reusable wrappers, like Vectors with different types inside
    for reusable_wrapper in &wrapper.reusable_wrappers {
        let file_name = match reusable_wrapper {
            ReusableWrapper::Vec(inner) => format!("vec_{}.swift", inner.name()),
            ReusableWrapper::Result(inner) => format!("result_{}.swift", inner.name()),
            ReusableWrapper::Option(inner) => format!("option_{}.swift", inner.name()),
        };
        let source_full_path = ffi_module_path.join(file_name);
        if SWIFT_CLASS_GENERATED
            .lock()
            .expect("Mutex lock failed during locking for reusable wrapper")
            .insert(source_full_path.clone())
        {
            create_file(reusable_wrapper.swift(), source_full_path);
        }
    }
}

pub(crate) fn swift_code_base() -> String {
    format!(
        r#"
import Foundation

public typealias bool = Bool

open class Opaque {{
    private var _self: UnsafeMutableRawPointer

    public required init(_ _self: UnsafeMutableRawPointer) {{
        self._self = _self
    }}

    open func rawPtr() -> UnsafeMutableRawPointer {{
        return self._self
    }}
}}

public class RustString: Opaque {{
    public func to_string() -> String {{
        let c_void_ptr = {RUST_STRING_DATA_FN_NAME}(self.rawPtr())
        let c_str_ptr = c_void_ptr!.assumingMemoryBound(to: UInt8.self)
        let c_str_len = {RUST_STRING_LEN_FN_NAME}(self.rawPtr())
        let bytes: UnsafeBufferPointer<UInt8> = UnsafeBufferPointer(start: c_str_ptr, count: Int(c_str_len))
        return String(bytes: bytes, encoding: .utf8)!
    }}

    deinit {{
        {RUST_STRING_DROP_FN_NAME}(self.rawPtr());
    }}
}}

public class RustError: Error {{
    private var rust_error: UnsafeMutableRawPointer
    private var source_error: UnsafeMutableRawPointer?
    private var desc: String

    public init(_ rust_error: UnsafeMutableRawPointer) {{
        self.rust_error = rust_error
        self.source_error = nil
        let rust_desc = RustString({RUST_ARC_DYN_ERR_DESC_FN_NAME}(rust_error));
        self.desc = rust_desc.to_string();
    }}

    public init(_ rust_error: UnsafeMutableRawPointer, _ source_ptr: UnsafeMutableRawPointer) {{
        self.rust_error = rust_error
        self.source_error = source_ptr
        let rust_desc = RustString({RUST_REF_DYN_ERR_DESC_FN_NAME}(source_ptr));
        self.desc = rust_desc.to_string();
    }}

    public func description() -> String {{
        return self.desc
    }}

    public func source() -> RustError? {{
        var source_dyn_err_ptr: UnsafeMutableRawPointer? = nil
        if self.source_error == nil {{
            source_dyn_err_ptr = {RUST_ARC_DYN_ERR_SOURCE_FN_NAME}(self.rust_error);
        }} else {{
            source_dyn_err_ptr = {RUST_REF_DYN_ERR_SOURCE_FN_NAME}(self.source_error);
        }}

        if source_dyn_err_ptr == nil {{
            return nil
        }} else {{
            let root_err_clone = {RUST_ARC_DYN_ERR_CLONE_FN_NAME}(self.rust_error);
            let source_error = RustError(root_err_clone!, source_dyn_err_ptr!)
            return source_error
        }}
    }}

    deinit {{
        if self.source_error != nil {{
            {RUST_REF_DYN_ERR_DROP_FN_NAME}(self.source_error!)
        }}
        {RUST_ARC_DYN_ERR_DROP_FN_NAME}(self.rust_error)
    }}
}}
"#
    )
}

// Marker to insert enum definitions
pub(crate) const ENUM_DEFINITIONS_MARKER: &str = "// ENUM_DEFINITIONS_MARKER";

pub(crate) fn swift_c_header_code_base() -> String {
    format!(
        r#"
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef int8_t i8;
typedef int16_t i16;
typedef int32_t i32;
typedef int64_t i64;

typedef float f32;
typedef double f64;

typedef size_t usize;

// ENUM_DEFINITIONS_MARKER

void* {RUST_STRING_DATA_FN_NAME}(void* self);
unsigned int {RUST_STRING_LEN_FN_NAME}(void* self);
void {RUST_STRING_DROP_FN_NAME}(void* self);

void* {SLICE_GET_PTR_FN_NAME}(void* self);
unsigned int {SLICE_GET_LEN_FN_NAME}(void* self);
void {SLICE_DROP_FN_NAME}(void* self);

void {RUST_ARC_DYN_ERR_DROP_FN_NAME}(void* self);
void* {RUST_ARC_DYN_ERR_DESC_FN_NAME}(void* self);
void* {RUST_ARC_DYN_ERR_SOURCE_FN_NAME}(void* self);
void* {RUST_ARC_DYN_ERR_CLONE_FN_NAME}(void* self);

void {RUST_REF_DYN_ERR_DROP_FN_NAME}(void* self);
void* {RUST_REF_DYN_ERR_DESC_FN_NAME}(void* self);
void* {RUST_REF_DYN_ERR_SOURCE_FN_NAME}(void* self);
"#
    )
}

pub(crate) fn swift_c_ffi_package_definition() -> String {
    r#"
// swift-tools-version: 5.5
import PackageDescription

let package = Package(
    name: "CFfiModule",
    products: [
        .library(name: "CFfiModule", targets: ["CFfiModule"]),
    ],
    targets: [
        .systemLibrary(name: "CFfiModule"),
    ]
)
"#
    .to_string()
}

pub(crate) fn swift_ffi_package_definition() -> String {
    r#"
// swift-tools-version: 5.5
import PackageDescription

let package = Package(
    name: "FfiModule",
    products: [
        .library(name: "FfiModule", targets: ["FfiModule"]),
    ],
    dependencies: [
        .package(path: "../CFfiModule"),
    ],
    targets: [
        .target(
            name: "FfiModule",
            dependencies: ["CFfiModule"]),
    ]
)
"#
    .to_string()
}

pub(crate) fn clang_module_map(package_name: impl Display) -> String {
    format!(
        r#"
module CFfiModule {{
    header "ffi_swift.h"
    link "{package_name}"
    export *
}}
"#
    )
}
