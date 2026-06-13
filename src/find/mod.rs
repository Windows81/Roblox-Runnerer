mod init_main;
pub mod util;

pub fn find_main(
    h_get_proc_id_dll: u64,
    dll_main_rva: u64,
) -> impl AsyncFn(Vec<std::string::String>) {
    init_main::find_main(h_get_proc_id_dll, dll_main_rva)
}
