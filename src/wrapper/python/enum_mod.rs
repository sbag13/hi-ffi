use crate::wrapper::EnumWrapper;
use crate::wrapper::python::ClassCode;
use std::collections::HashMap;

pub(crate) fn gen_enum_class(enum_wrapper: &EnumWrapper) -> ClassCode {
    let enum_name = &enum_wrapper.name;

    // Generate enum variants
    let variants: Vec<String> = enum_wrapper
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.name;
            let variant_value = variant.discriminant.as_ref().unwrap();
            format!("    {} = {}", variant_name, variant_value)
        })
        .collect();

    let enum_definition = format!(
        r#"class {}(IntEnum):
{}
    @classmethod
    def from_ffi(cls, value):
        return cls(value)
    
    def to_ffi(self):
        return self.value
"#,
        enum_name,
        variants.join("\n")
    );

    let mut imports = HashMap::new();
    imports.insert("enum".to_string(), "from enum import IntEnum".to_string());

    ClassCode {
        header: String::new(), // No header needed for enum
        body: enum_definition,
        name: enum_name.to_string(),
        imports,
    }
}
