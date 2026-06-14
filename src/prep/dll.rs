use std::path::Path;

use windows::{
    Win32::{
        Foundation::{HANDLE, HMODULE},
        System::LibraryLoader::{
            GetProcAddress, LOAD_LIBRARY_SEARCH_APPLICATION_DIR, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32,
            LOAD_LIBRARY_SEARCH_USER_DIRS, LoadLibraryExW,
        },
    },
    core::HSTRING,
};

use crate::offsets;

pub fn load(dir_path: &Path, dll_name: &str) -> u64 {
    let dll_path = dir_path.join(dll_name);

    let flags = LOAD_LIBRARY_SEARCH_DEFAULT_DIRS
        | LOAD_LIBRARY_SEARCH_SYSTEM32
        | LOAD_LIBRARY_SEARCH_APPLICATION_DIR
        | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR
        | LOAD_LIBRARY_SEARCH_USER_DIRS;

    // Load the patched DLL
    unsafe {
        LoadLibraryExW(
            &HSTRING::from(dll_path.as_os_str()),
            HANDLE(std::ptr::null_mut()),
            flags,
        )
    }
    .unwrap()
    .0 as u64
}
