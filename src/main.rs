pub mod find;

pub mod macros;
pub mod patch;

use std::ffi::c_char;
use std::{ffi::CString, path::Path};

use cstring_array::CStringArray;
use windows::{
    Win32::{
        Foundation::{HANDLE, HMODULE},
        System::{
            Environment::SetCurrentDirectoryW,
            LibraryLoader::{
                GetProcAddress, LOAD_LIBRARY_SEARCH_APPLICATION_DIR,
                LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
                LOAD_LIBRARY_SEARCH_SYSTEM32, LOAD_LIBRARY_SEARCH_USER_DIRS, LoadLibraryExW,
                LoadLibraryW,
            },
        },
    },
    core::HSTRING,
};

/// Constructs a QString on the stack and calls fromAscii.
/// Returns a pointer to the stack-allocated buffer.
fn construct_qstring(qt5core: HMODULE, ptr: *mut *mut u8, data: &str) -> *mut *mut u8 {
    let from_ascii = getproc_offset_func!(
        qt5core,
        windows::core::s!("?fromAscii@QString@@SA?AV1@PEBDH@Z"),
        *mut *mut u8,
        (*mut *mut u8, *const i8, i32),
        "C"
    );
    let c_data = CString::new(data).unwrap();
    // Pass the address of our stack buffer
    from_ascii(ptr, c_data.as_ptr(), data.len() as i32)
}

/// Destructs the QString internal state but DOES NOT free the object memory itself
/// because we allocated it on the stack.
unsafe fn destruct_qstring_no_free(qt5core: HMODULE, object: *mut *mut u8) -> *mut u8 {
    if unsafe { *object }.is_null() {
        return std::ptr::null_mut();
    };
    let destruct = getproc_offset_func!(
        qt5core,
        windows::core::s!("??1QString@@QEAA@XZ"),
        *mut u8,
        (*mut *mut u8),
        "C"
    );
    destruct(object)
}

fn test_qstring(internal_data_ptr: &*mut u8) {
    if internal_data_ptr.is_null() {
        return;
    };
    unsafe {
        // Add 0x18 bytes to get to the wide string data
        let wide_str_ptr = internal_data_ptr.add(0x18) as *const u16;

        // Calculate length of wide string
        let mut len = 0;
        while *wide_str_ptr.offset(len) != 0 {
            len += 1;
        }

        if len > 0 {
            let slice = std::slice::from_raw_parts(wide_str_ptr, len as usize);
            if let Ok(s) = String::from_utf16(slice) {
                println!("Qt Application Path: {}", s);
            }
        }
    }
}

/// Mimics change_qapplicationdir using stack-allocated QString
fn change_qapplication_dir(qt5core: HMODULE, dll_path_str: &Path) {
    let set_application_file_path = getproc_offset_func!(
        qt5core,
        windows::core::s!("?setApplicationFilePath@QCoreApplicationPrivate@@SAXAEBVQString@@@Z"),
        *mut u8,
        (&*mut u8),
        "C"
    );
    let mut internal_data_ptr = Box::<_>::new(std::ptr::null_mut());
    construct_qstring(
        qt5core,
        &mut *internal_data_ptr,
        dll_path_str.to_str().unwrap(),
    );
    test_qstring(&*internal_data_ptr);
    set_application_file_path(&*internal_data_ptr);
}

/// Loads the required dependent DLLs from the specified directory.
fn load_dependent_user_libraries(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let names = [
        "WebView2Loader.dll",
        "libGLESv2.dll",
        "libfbxsdk.dll",
        "msvcp140.dll",
        "sgCore.dll",
        "vcruntime140.dll",
        "vcruntime140_1.dll",
    ];
    unsafe {
        for name in names {
            let path = dir.join(name);
            let res = LoadLibraryW(&HSTRING::from(path.as_os_str()))?;
            println!(" {} {:?}", name, res);
        }
    }
    Ok(())
}

fn main() {
    //let dir_path = Path::new(r"C:\Users\USER\Projects\FilteringDisabled\Roblox\v463\Studio\");
    let dir_path = Path::new(r"C:\Program Files\RobloxStudio548\");
    let exe_path = dir_path.join("RobloxStudioBeta.exe");
    let dll_path = dir_path.join("RobloxStudioBeta.dll");

    let qt5core_path = dir_path.join("Qt5Core.dll");

    let flags = LOAD_LIBRARY_SEARCH_DEFAULT_DIRS
        | LOAD_LIBRARY_SEARCH_SYSTEM32
        | LOAD_LIBRARY_SEARCH_APPLICATION_DIR
        | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR
        | LOAD_LIBRARY_SEARCH_USER_DIRS;

    // Load Qt5Core.dll (Handle is dropped immediately as it's not actively used in main)
    let qt5core = unsafe {
        LoadLibraryExW(
            &HSTRING::from(qt5core_path.as_os_str()),
            HANDLE(std::ptr::null_mut()),
            flags,
        )
    }
    .unwrap();

    let _ = unsafe { SetCurrentDirectoryW(&HSTRING::from(dir_path.as_os_str())) };
    change_qapplication_dir(qt5core, &dll_path);

    let dll_main = patch::patch_dll(&exe_path, &dll_path).unwrap() as u64;
    let _ = load_dependent_user_libraries(&dir_path);

    // Load the patched DLL
    let h_get_proc_id_dll = unsafe {
        LoadLibraryExW(
            &HSTRING::from(dll_path.as_os_str()),
            HANDLE(std::ptr::null_mut()),
            flags,
        )
    }
    .unwrap()
    .0 as u64;

    let main = find::find_main(h_get_proc_id_dll, dll_main);
    main();
}
