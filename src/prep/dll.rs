use std::path::Path;

use windows::{
    Win32::{
        Foundation::HANDLE,
        System::LibraryLoader::{
            LOAD_LIBRARY_FLAGS, LOAD_LIBRARY_SEARCH_APPLICATION_DIR,
            LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
            LOAD_LIBRARY_SEARCH_SYSTEM32, LOAD_LIBRARY_SEARCH_USER_DIRS, LoadLibraryExW,
        },
    },
    core::HSTRING,
};

pub const SEARCH_FLAGS: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(
    LOAD_LIBRARY_SEARCH_DEFAULT_DIRS.0
        | LOAD_LIBRARY_SEARCH_SYSTEM32.0
        | LOAD_LIBRARY_SEARCH_APPLICATION_DIR.0
        | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR.0
        | LOAD_LIBRARY_SEARCH_USER_DIRS.0,
);

/// Loads `name` from `dir` and returns its base address.
pub fn load(dir: &Path, name: &str) -> windows::core::Result<u64> {
    let path = dir.join(name);
    let module = unsafe {
        LoadLibraryExW(
            &HSTRING::from(path.as_os_str()),
            HANDLE(std::ptr::null_mut()),
            SEARCH_FLAGS,
        )
    }?;
    Ok(module.0 as u64)
}
