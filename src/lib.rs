use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Once;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use syn::{Item, parse_macro_input};
use translator::translate;
use wrapper::Wrapper;
use wrapper::base::rust_code_base;

use crate::wrapper::ReusableWrapper;

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

static RUST_CODE_BASE_TOKENS_GENERATED: Once = Once::new();

// Tokens of wrapper that should be generated only once
static RUST_WRAPPER_TOKENS_GENERATED: LazyLock<Mutex<HashSet<ReusableWrapper>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static WAITING_FOR_WRAPPERS: LazyLock<Mutex<HashMap<String, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[proc_macro_attribute]
pub fn ffi(_attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_ffi(input)
}

fn handle_ffi(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    let wrapper = match translate(input.clone()) {
        Ok(wrapper) => wrapper,
        Err(wrapper_name) => {
            WAITING_FOR_WRAPPERS
                .lock()
                .unwrap()
                .entry(wrapper_name.0.to_string())
                .or_default()
                .push(input.to_token_stream().to_string());

            let wrapper_name_token: TokenStream2 = wrapper_name.0.parse().unwrap();
            return quote! { impl #wrapper_name_token {} }.into();
        }
    };

    let waiting_tokens = { WAITING_FOR_WRAPPERS.lock().unwrap().remove(&wrapper.name()) }.map(
        |waiting_definitions| {
            waiting_definitions
                .iter()
                .map(|tokens: &String| handle_ffi(tokens.parse().unwrap()))
                .collect::<TokenStream>()
        },
    );

    write_rust_code(&wrapper);
    #[cfg(feature = "cpp")]
    cpp::write_cpp_code(&wrapper);
    #[cfg(feature = "swift")]
    swift::write_swift_code(&wrapper);
    #[cfg(feature = "python")]
    python::write_python_code(&wrapper);

    let mut tokens: TokenStream2 = (&wrapper).into();

    if let Some(waiting_tokens) = waiting_tokens {
        tokens.extend(TokenStream2::from(waiting_tokens));
    }

    RUST_CODE_BASE_TOKENS_GENERATED.call_once(|| tokens.extend(rust_code_base()));

    // Generate reusable wrappers, like Vectors with different types inside
    for reusable_wrapper in &wrapper.reusable_wrappers {
        if RUST_WRAPPER_TOKENS_GENERATED
            .lock()
            .expect("Mutex lock failed during locking for reusable wrapper tokens")
            .insert(reusable_wrapper.clone())
        {
            let wrapper_tokens: TokenStream2 = reusable_wrapper.into();
            tokens.extend(wrapper_tokens);
        }
    }

    tokens.into()
}

static RUST_WRAPPER_FILE_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
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

    if RUST_WRAPPER_FILE_GENERATED
        .lock()
        .expect("Mutex lock failed")
        .insert(full_file_path.clone())
    {
        create_file(rust_tokens, full_file_path);
    } else {
        append_to_file(&rust_tokens, full_file_path);
    }

    // Generate reusable wrappers, like Vectors with different types inside
    for reusable_wrapper in &wrapper.reusable_wrappers {
        let file_name = match reusable_wrapper {
            ReusableWrapper::Vec(inner) => format!("vec_{}.rs", inner.name()),
            ReusableWrapper::Result(inner) => format!("result_{}.rs", inner.name()),
            ReusableWrapper::Option(inner) => format!("option_{}.rs", inner.name()),
        };
        let full_file_path = rust_path.join(&file_name);
        if RUST_WRAPPER_FILE_GENERATED
            .lock()
            .expect("Mutex lock failed during locking for reusable wrapper")
            .insert(full_file_path.clone())
        {
            let tokens: TokenStream2 = reusable_wrapper.into();
            create_file(tokens, full_file_path);
        }
    }
}

#[cfg(any(feature = "swift", feature = "cpp"))]
fn insert_after(marker: &str, content: impl Display, path: impl AsRef<Path>) {
    let content = format!("{}", content);
    let file_content = std::fs::read_to_string(path.as_ref()).expect("Unable to read file");
    let new_content = file_content.replace(marker, &format!("{}\n{}", marker, content));
    create_file(new_content, path);
}

#[cfg(feature = "cpp")]
fn insert_after_if_not_present(marker: &str, content: impl Display, path: impl AsRef<Path>) {
    let content = content.to_string();
    let file_content = std::fs::read_to_string(path.as_ref()).expect("Unable to read file");
    if !file_content.contains(&content) {
        let new_content = file_content.replace(marker, &format!("{}\n{}", marker, content));
        create_file(new_content, path);
    }
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
