use crate::wrapper::base::*;
use std::fmt::Display;

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

"#
    )
}

pub(crate) fn swift_c_header_code_base() -> String {
    format!(
        r#"
#include <stdint.h>
#include <stdbool.h>

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

void* {RUST_STRING_DATA_FN_NAME}(void* self);
int {RUST_STRING_LEN_FN_NAME}(void* self);
void {RUST_STRING_DROP_FN_NAME}(void* self);
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
