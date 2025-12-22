use crate::wrapper::EnumWrapper;
use crate::wrapper::swift::SwiftCode;

pub(crate) fn gen_enum_code(enum_wrapper: &EnumWrapper) -> SwiftCode {
    let enum_name = &enum_wrapper.name;
    let swift_variants = enum_wrapper
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.name;
            let variant_value = variant.discriminant.as_ref().unwrap();
            format!("    case {} = {}", variant_name, variant_value)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let c_variants = enum_wrapper
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.name;
            let variant_value = variant.discriminant.as_ref().unwrap();
            format!("    {} = {}", variant_name, variant_value)
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let enum_declaration = format!(
        r#"public enum {enum_name}: Int32 {{
{swift_variants}
}}
"#,
    );

    let c_header = format!(
        "enum {enum_name} {{
{c_variants}
}};"
    );

    SwiftCode::Enum {
        source: enum_declaration,
        header: c_header,
    }
}
