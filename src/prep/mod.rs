use std::path::Path;

use windows::{Win32::System::Environment::SetCurrentDirectoryW, core::HSTRING};

use crate::offsets;

mod dll;
mod patch;
mod qt;

const STUDIO_EXE: &str = "RobloxStudioBeta.exe";
const STUDIO_DLL: &str = "RobloxStudioBeta.dll";

pub fn load(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let dll_main_rva = patch::patch_dll(dir, STUDIO_EXE, STUDIO_DLL)?;

    // Studio resolves a lot of its content relative to the cwd.
    unsafe { SetCurrentDirectoryW(&HSTRING::from(dir.as_os_str()))? };
    qt::prepare(dir, STUDIO_DLL)?;

    let base = dll::load(dir, STUDIO_DLL)?;
    let _offsets = offsets::find(base, dll_main_rva as u64);

    Ok(())
}
