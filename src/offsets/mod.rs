pub mod datamodel;
mod init_main;
pub mod util;

pub struct Offsets {
    pub init_main: init_main::InitMain,
    pub datamodel: datamodel::DataModelFns,
}

pub fn find(base: u64, dll_main_rva: u64) -> Offsets {
    Offsets {
        init_main: init_main::find(base, dll_main_rva),
        datamodel: datamodel::find(base),
    }
}

/// Finds everything and runs the CRT init so the DLL is actually usable.
pub fn prepare(base: u64, dll_main_rva: u64) -> Offsets {
    let offsets = find(base, dll_main_rva);
    offsets.init_main.init_crt();
    offsets
}
