use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Once;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use syn::{Item, parse_macro_input};
use translator::translate;
use wrapper::Wrapper;
use wrapper::base::rust_code_base;

#[cfg(feature = "cpp")]
mod cpp;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "swift")]
mod swift;
mod translator;
mod wrapper;

const EXPORTED_SYMBOLS_PREFIX: &str = "hiFfi__";
const GEN_CODE_DIR: &str = "./generated_code/";
const RUST_CODE_DIR: &str = "rust/";
#[cfg(feature = "cpp")]
const CPP_CODE_DIR: &str = "cpp/";

static RUST_CODE_BASE_GENERATED: Once = Once::new();

#[proc_macro_attribute]
pub fn ffi(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    let wrapper = translate(input);

    write_rust_code(&wrapper);
    #[cfg(feature = "cpp")]
    cpp::write_cpp_code(&wrapper);
    #[cfg(feature = "swift")]
    swift::write_swift_code(&wrapper);
    #[cfg(feature = "python")]
    python::write_python_code(&wrapper);

    let mut tokens: TokenStream2 = (&wrapper).into();

    RUST_CODE_BASE_GENERATED.call_once(|| tokens.extend(rust_code_base()));

    tokens.into()
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
    writeln!(file, "\n{}", content).expect("Unable to write data");
}

#[cfg(feature = "python")]
fn prepend_to_file(content: impl Display, path: impl AsRef<Path>) {
    let file_content = std::fs::read_to_string(path.as_ref()).expect("Unable to read file");
    let trimmed_content = content.to_string().trim().to_string();
    let trimmed_file_content = file_content.trim().to_string();
    let new_content = format!("{}\n{}", trimmed_content, trimmed_file_content);
    create_file(new_content, path);
}

#[cfg(any(feature = "swift", feature = "cpp"))]
fn prepend_each_line_with_n_tabs(s: &str, n: usize) -> String {
    let tabs = "    ".repeat(n);
    s.lines()
        .map(|line| format!("{}{}", tabs, line))
        .collect::<Vec<_>>()
        .join("\n")
}
