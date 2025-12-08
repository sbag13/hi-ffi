use std::{fmt::Display, fs::OpenOptions, io::Write, path::Path, sync::Once};

#[cfg(feature = "cpp")]
use cpp::cpp_code_base;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
#[cfg(feature = "swift")]
use swift::*;
use syn::{parse_macro_input, Item};
use translator::translate;
#[cfg(feature = "cpp")]
use wrapper::cpp::CppHeader;
use wrapper::{base::rust_code_base, Wrapper};

#[cfg(feature = "cpp")]
mod cpp;
#[cfg(feature = "swift")]
mod swift;
mod translator;
mod wrapper;

const EXPORTED_SYMBOLS_PREFIX: &str = "__hiFfi__";
const GEN_CODE_DIR: &str = "./generated_code/";
const RUST_CODE_DIR: &str = "rust/";
#[cfg(feature = "cpp")]
const CPP_CODE_DIR: &str = "cpp/";
#[cfg(feature = "swift")]
const SWIFT_CODE_DIR: &str = "swift/";

static RUST_CODE_BASE_GENERATED: Once = Once::new();
#[cfg(feature = "swift")]
static SWIFT_C_HEADER_RECREATED: Once = Once::new();

#[proc_macro_attribute]
pub fn ffi(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    let wrapper = translate(input);

    write_rust_code(&wrapper);
    #[cfg(feature = "cpp")]
    write_cpp_code(&wrapper);
    #[cfg(feature = "swift")]
    write_swift_code(&wrapper);

    let mut tokens: TokenStream2 = (&wrapper).into();

    RUST_CODE_BASE_GENERATED.call_once(|| tokens.extend(rust_code_base()));

    tokens.into()
}

#[cfg(feature = "swift")]
static SWIFT_CLASS_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

#[cfg(feature = "swift")]
fn write_swift_code(wrapper: &Wrapper) {
    use crate::wrapper::SwiftCode;
    use wrapper::swift::class_definition::gen_empty_class_definition;

    let swift_path = Path::new(GEN_CODE_DIR).join(SWIFT_CODE_DIR);
    std::fs::create_dir_all(&swift_path).expect("Unable to create swift directory");

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

    append_to_file(swift_code.header(), swift_header_path);

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
        SwiftCode::Function { source, .. } => create_file(source, &source_path),
    }
}

static RUST_STRUCT_WRAPPER_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn write_rust_code(wrapper: &Wrapper) {
    let rust_path = Path::new(GEN_CODE_DIR).join(RUST_CODE_DIR);
    std::fs::create_dir_all(&rust_path).expect("Unable to create rust directory");

    let rust_base_path = rust_path.join("base.rs");
    // if !code_base_path.exists() { // TODO: uncomment when stable implementation is ready
    create_file(rust_code_base(), rust_base_path);
    // }

    let file_name = format!("{}.rs", wrapper.name());
    let full_file_path = rust_path.join(&file_name);
    let rust_tokens: TokenStream2 = wrapper.into();

    if RUST_STRUCT_WRAPPER_GENERATED
        .lock()
        .expect("Mutex lock failed")
        .insert(full_file_path.clone())
    {
        create_file(rust_tokens, full_file_path);
    } else {
        append_to_file(&rust_tokens, full_file_path);
    }
}

#[cfg(feature = "cpp")]
fn write_cpp_code(wrapper: &Wrapper) {
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

    let crate::wrapper::CppFiles { header, source } = wrapper.cpp();
    write_header(header, header_full_path);

    if let Some(source) = source {
        create_file(source, source_full_path);
    }
}

#[cfg(feature = "cpp")]
static CPP_CLASS_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

#[cfg(feature = "cpp")]
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

#[cfg(any(feature = "swift", feature = "cpp"))]
fn insert_after(marker: &str, content: impl Display, path: impl AsRef<Path>) {
    let content = format!("{}", content);
    let file_content = std::fs::read_to_string(path.as_ref()).expect("Unable to read file");
    let new_content = file_content.replace(marker, &format!("{}\n{}", marker, content));
    create_file(new_content, path);
}

fn create_file(content: impl Display, path: impl AsRef<Path>) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Unable to open file");
    writeln!(file, "{}", content).expect("Unable to write data");
}

fn append_to_file(content: impl Display, path: impl AsRef<Path>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Unable to open file");
    writeln!(file, "{}", content).expect("Unable to write data");
}

#[cfg(any(feature = "swift", feature = "cpp"))]
fn prepend_each_line_with_n_tabs(s: &str, n: usize) -> String {
    let tabs = "    ".repeat(n);
    s.lines()
        .map(|line| format!("{}{}", tabs, line))
        .collect::<Vec<_>>()
        .join("\n")
}
