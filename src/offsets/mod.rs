pub mod datamodel;
mod init_main;
pub mod util;

pub struct CompositeStruct {
    pub init_main: init_main::InitMain,
    pub datamodel: datamodel::DataModel,
}

pub fn find(rōblox_dll_ptr: u64, dll_main_rva: u64) -> CompositeStruct {
    CompositeStruct {
        init_main: init_main::find(rōblox_dll_ptr, dll_main_rva),
        datamodel: datamodel::find(rōblox_dll_ptr),
    }
}

pub fn prepare_rōblox(rōblox_dll_ptr: u64, dll_main_rva: u64) -> CompositeStruct {
    let result = find(rōblox_dll_ptr, dll_main_rva);
    result.init_main.prep_rōblox();
    result
}
