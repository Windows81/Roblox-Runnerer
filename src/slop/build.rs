//! Compiles the Win32 resource script (`WindowsClient.rc`) so the icon,
//! cursor, dialog templates (`IDD_RBXWEBVIEW`, `IDD_UPLOADVIDEODIALOG`),
//! accelerator table (`IDR_GAME_ACCELERATOR`) and string table are embedded,
//! exactly as the original `.vcxproj` did.
//!
//! Drop the original `WindowsClient.rc` (and its referenced `.ico`/`.cur`/HTML
//! assets) next to this file to enable resource embedding.

fn main() {
    if std::path::Path::new("WindowsClient.rc").exists() {
        embed_resource::compile("WindowsClient.rc", embed_resource::NONE);
    } else {
        println!(
            "cargo:warning=WindowsClient.rc not found; building without embedded \
             resources (icons/dialogs/accelerators will be unavailable)."
        );
    }
}
