use std::path::Path;

use windows::{Win32::System::Environment::SetCurrentDirectoryW, core::HSTRING};

use crate::slop;

mod dll;
mod libs;
mod patch;
mod qt;

pub fn load(dir_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let _ = unsafe { SetCurrentDirectoryW(&HSTRING::from(dir_path.as_os_str())) };
    let dll_main_rva = patch::patch_dll(dir_path, "RobloxStudioBeta.exe", "RobloxStudioBeta.dll")?;
    slop::main(dll_main_rva);
    qt::load(dir_path, "RobloxStudioBeta.dll")?;
    //libs::load(dir_path)?;
    let main = dll::load(dir_path, "RobloxStudioBeta.dll", dll_main_rva as u64)?;

    let exe_path = dir_path.join("RobloxStudioBeta.exe");
    //futures::executor::block_on(main(vec![exe_path.to_str().unwrap().into(), "67".into()]));
    Ok(())
}
