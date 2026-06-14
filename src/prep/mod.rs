use std::path::Path;

use windows::{Win32::System::Environment::SetCurrentDirectoryW, core::HSTRING};

use crate::{offsets, slop};

mod dll;
mod patch;
mod qt;

pub fn load(dir_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let dll_main_rva = patch::patch_dll(dir_path, "RobloxStudioBeta.exe", "RobloxStudioBeta.dll")?;

    let _ = unsafe { SetCurrentDirectoryW(&HSTRING::from(dir_path.as_os_str())) };
    qt::prepare(dir_path, "RobloxStudioBeta.dll")?;

    let rōblox_dll_ptr = dll::load(dir_path, "RobloxStudioBeta.dll");
    let routines = offsets::find(rōblox_dll_ptr, dll_main_rva as u64);
    let data_model_funcs = offsets::datamodel::find(rōblox_dll_ptr);
    slop::main(routines.datamodel);

    let exe_path = dir_path.join("RobloxStudioBeta.exe");
    //futures::executor::block_on(main(vec![exe_path.to_str().unwrap().into(), "67".into()]));
    Ok(())
}
