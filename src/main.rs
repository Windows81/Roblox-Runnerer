pub mod macros;
pub mod offsets;
pub mod prep;
pub mod structs;
// pub mod slop;

use std::path::PathBuf;

const DEFAULT_STUDIO_DIR: &str = r"C:\Users\USER\Projects\FilteringDisabled\Roblox\v463\Studio\";
// const DEFAULT_STUDIO_DIR: &str = r"C:\Program Files\RobloxStudio548\";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let studio_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_STUDIO_DIR));

    prep::load(&studio_dir)
}
