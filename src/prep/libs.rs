use std::path::Path;

use windows::{Win32::System::LibraryLoader::LoadLibraryW, core::HSTRING};

/// Loads the required dependent DLLs from the specified directory.
pub fn load(dir_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
            let path = dir_path.join(name);
            let res = LoadLibraryW(&HSTRING::from(path.as_os_str()))?;
            println!(" {} {:?}", name, res);
        }
    }
    Ok(())
}
