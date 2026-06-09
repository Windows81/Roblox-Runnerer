mod init_main;
pub mod util;
use crate::get_func_addr;
use cstring_array::CStringArray;
use util::{load_calls_to_struct, load_decoder};

pub fn find_main(h_get_proc_id_dll: u64, dll_main_rva: u64) -> impl FnOnce() {
    init_main::find_main(h_get_proc_id_dll, dll_main_rva)
}
