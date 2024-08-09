use crate::wrapper::base::*;

pub(crate) fn cpp_code_base() -> String {
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
