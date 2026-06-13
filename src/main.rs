pub mod find;
pub mod prep;
pub mod slop;

pub mod macros;

use std::path::Path;

use windows::Win32::Foundation::HMODULE;

fn main() {
    let dir_path = Path::new(r"C:\Users\USER\Projects\FilteringDisabled\Roblox\v463\Studio\");
    //let dir_path = Path::new(r"C:\Program Files\RobloxStudio548\");

    prep::load(dir_path);
}
