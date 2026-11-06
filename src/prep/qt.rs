use std::{ffi::CString, path::Path};

use windows::{
    Win32::{
        Foundation::{HANDLE, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryExW},
    },
    core::{HSTRING, s},
};

use super::dll::SEARCH_FLAGS;
use crate::getproc_offset_func;

/// Builds a QString via `QString::fromAscii`, writing its d-pointer into `out`.
fn qstring_from_ascii(qt5core: HMODULE, out: *mut *mut u8, text: &str) -> *mut *mut u8 {
    let from_ascii = getproc_offset_func!(
        qt5core,
        s!("?fromAscii@QString@@SA?AV1@PEBDH@Z"),
        *mut *mut u8,
        (*mut *mut u8, *const i8, i32),
        "C"
    );
    let c_text = CString::new(text).unwrap();
    from_ascii(out, c_text.as_ptr(), text.len() as i32)
}

/// Runs `~QString` without freeing the QString itself (we don't own that allocation).
#[allow(dead_code)]
unsafe fn qstring_drop_in_place(qt5core: HMODULE, object: *mut *mut u8) -> *mut u8 {
    if unsafe { *object }.is_null() {
        return std::ptr::null_mut();
    }
    let dtor = getproc_offset_func!(
        qt5core,
        s!("??1QString@@QEAA@XZ"),
        *mut u8,
        (*mut *mut u8),
        "C"
    );
    dtor(object)
}

/// Debug helper: dumps the UTF-16 contents of a QString d-pointer.
fn dump_qstring(d: *mut u8) {
    if d.is_null() {
        return;
    }
    unsafe {
        // QArrayData header is 0x18 bytes, the UTF-16 payload follows.
        let chars = d.add(0x18) as *const u16;
        let mut len = 0;
        while *chars.add(len) != 0 {
            len += 1;
        }
        if let Ok(s) = String::from_utf16(std::slice::from_raw_parts(chars, len)) {
            println!("Qt application path: {s}");
        }
    }
}

/// Points Qt's application file path at the Studio binary so it finds its
/// plugins/resources next to it instead of next to our exe.
fn set_application_path(qt5core: HMODULE, studio_path: &Path) {
    let set_application_file_path = getproc_offset_func!(
        qt5core,
        s!("?setApplicationFilePath@QCoreApplicationPrivate@@SAXAEBVQString@@@Z"),
        *mut u8,
        (&*mut u8),
        "C"
    );
    let mut d = Box::new(std::ptr::null_mut());
    qstring_from_ascii(qt5core, &mut *d, studio_path.to_str().unwrap());
    dump_qstring(*d);
    set_application_file_path(&*d);
}

pub fn prepare(dir: &Path, studio_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let qt5core = unsafe {
        LoadLibraryExW(
            &HSTRING::from(dir.join("Qt5Core.dll").as_os_str()),
            HANDLE(std::ptr::null_mut()),
            SEARCH_FLAGS,
        )
    }?;

    set_application_path(qt5core, &dir.join(studio_name));
    Ok(())
}
