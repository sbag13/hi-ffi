use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, Once};

use crate::wrapper::python::{FunctionCode, PythonFiles};
use crate::{GEN_CODE_DIR, ReusableWrapper, Wrapper, append_to_file, create_file, prepend_to_file};

pub const PYTHON_LIB_GETTER_NAME: &str = "get_ffi_lib";
pub const FFI_INIT_FUNCTION_NAME: &str = "ffi_init";
static PYTHON_MODULE_RECREATED: Once = Once::new();

static MAIN_FILE_IMPORTS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static PER_CLASS_IMPORTS: LazyLock<Mutex<HashMap<String, HashMap<String, String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static WRAPPER_GENERATED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub(crate) fn write_python_code(wrapper: &Wrapper) {
    let python_path = Path::new(GEN_CODE_DIR).join("python_ffi/");

    let code_base_path = python_path.join("__init__.py");

    PYTHON_MODULE_RECREATED.call_once(|| {
        let _ = std::fs::remove_dir_all(&python_path);

        std::fs::create_dir_all(&python_path).expect("Unable to create python directory");
        create_file(python_code_base(), &code_base_path);
        let global_state_path = python_path.join("global_state.py");
        create_file(global_state(), global_state_path);
    });

    let PythonFiles { fn_code, class_mod } = wrapper.python();

    if let Some(FunctionCode { body, imports }) = fn_code {
        let mut locked_imports = MAIN_FILE_IMPORTS.lock().expect("Mutex lock failed");
        let imports_to_add = imports
            .into_iter()
            .filter(|(k, _)| !locked_imports.contains_key(k))
            .collect::<HashMap<_, _>>();
        prepend_to_file(
            imports_to_add
                .values()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            &code_base_path,
        );
        locked_imports.extend(imports_to_add);

        append_to_file(body, &code_base_path);
    }

    if let Some(class_mod) = class_mod {
        let class_mod_path = python_path.join(format!("{}.py", class_mod.name));
        if !class_mod_path.exists() {
            create_file(class_mod.header, &class_mod_path);
        }

        let mut locked_per_class_imports = PER_CLASS_IMPORTS.lock().expect("Mutex lock failed");
        let imports_for_class = locked_per_class_imports
            .entry(class_mod.name.clone())
            .or_default();
        let imports_to_add = class_mod
            .imports
            .into_iter()
            .filter(|(k, _)| !imports_for_class.contains_key(k))
            .collect::<HashMap<_, _>>();
        prepend_to_file(
            imports_to_add
                .values()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            &class_mod_path,
        );
        imports_for_class.extend(imports_to_add);

        append_to_file(class_mod.body, &class_mod_path);
    }

    for reusable_wrapper in &wrapper.reusable_wrappers {
        let file_name = match reusable_wrapper {
            ReusableWrapper::Vec(inner) => format!("vec_{}.py", inner.name()),
        };
        let file_path = python_path.join(file_name);
        if WRAPPER_GENERATED
            .lock()
            .expect("Mutex lock failed")
            .insert(file_path.clone())
        {
            create_file(reusable_wrapper.python(), &file_path);
        }
    }
}

fn global_state() -> String {
    format!(
        r#"import ctypes

hi_ffi_lib = None


def {FFI_INIT_FUNCTION_NAME}(lib: ctypes.CDLL):
    global hi_ffi_lib
    hi_ffi_lib = lib

def {PYTHON_LIB_GETTER_NAME}():
    global hi_ffi_lib
    return hi_ffi_lib

class RustString:
    def __init__(self, ptr):
        self._self_ptr = ptr

    def py_str(self) -> str:
        data = ctypes.c_char_p(get_ffi_lib().hiFfi__rust_string_data(self._self_ptr))
        length = get_ffi_lib().hiFfi__rust_string_len(self._self_ptr)
        return data.value[0:length].decode("utf-8") if length != 0 else ""

    def __del__(self):
        get_ffi_lib().hiFfi__rust_string_drop(self._self_ptr)

class FfiSlice:
    def __init__(self, ptr):
        self._self_ptr = ptr

    def py_str(self) -> str:
        data = ctypes.c_char_p(get_ffi_lib().hiFfi__slice_ptr(self._self_ptr))
        length = get_ffi_lib().hiFfi__slice_len(self._self_ptr)
        return data.value[0:length].decode("utf-8") if length != 0 else ""

    def __del__(self):
        get_ffi_lib().hiFfi__slice_drop(self._self_ptr)
"#
    )
}

pub fn python_code_base() -> String {
    format!(
        r#"from .global_state import {PYTHON_LIB_GETTER_NAME}, {FFI_INIT_FUNCTION_NAME}, RustString"#
    )
}
