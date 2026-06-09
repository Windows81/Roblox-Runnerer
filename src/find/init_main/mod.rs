mod scrt_init;
mod win_main;

use std::ffi::CString;

use super::util::load_decoder;
use crate::get_func_addr;
use cstring_array::CStringArray;

pub fn find_main(h_get_proc_id_dll: u64, dll_main_rva: u64) -> impl FnOnce() {
    let dll_main_ptr = h_get_proc_id_dll + dll_main_rva;

    // According to Studio v548
    // 00007FF774162E28 | 48:83EC 28               | sub rsp,28
    // 00007FF774162E2C | E8 230C0000              | call <robloxstudiobeta.__security_init_cookie>
    // 00007FF774162E31 | 48:83C4 28               | add rsp,28
    // 00007FF774162E35 | E9 7AFEFFFF              | jmp <robloxstudiobeta.__scrt_common_main_seh>
    // This leads to the head of `__scrt_common_main_seh`.
    let scrt_common_main_seh = {
        let mut decoder = load_decoder(dll_main_ptr + 0x0D, 0x05);
        let jmp_instruction = decoder.decode();
        jmp_instruction.near_branch_target()
    };

    let (scrt_init_map, scrt_init_callers) = scrt_init::get(scrt_common_main_seh);
    println!("{:?}", scrt_init_map);

    let (__xi_z, __xi_a) = {
        // Gets the addresses of the two `lea` instructions prior to the call to `_initterm_e`.
        let ins_size = 0x7;
        let read_size_before = ins_size * 2;
        let mut decoder = load_decoder(
            get_func_addr!(scrt_init_callers._initterm_e) - read_size_before,
            read_size_before,
        );
        (
            decoder.decode().memory_displacement64() as _, // __xi_z
            decoder.decode().memory_displacement64() as _, // __xi_a
        )
    };

    let (__xc_z, __xc_a) = {
        // Gets the addresses of the two `lea` instructions prior to the call to `_initterm`.
        let ins_size = 0x7;
        let read_size_before = ins_size * 2;
        let mut decoder = load_decoder(
            get_func_addr!(scrt_init_callers._initterm) - read_size_before,
            read_size_before,
        );
        (
            decoder.decode().memory_displacement64() as _, // __xi_z
            decoder.decode().memory_displacement64() as _, // __xi_a
        )
    };

    let win_main_addr = get_func_addr!(scrt_init_map.win_main);
    let (win_main_map, win_main_callers) = win_main::get(win_main_addr);

    let result = move || {
        // (scrt_init_map.__scrt_common_main_seh)();
        //return;
        println!("{:?}", scrt_init_map.__scrt_initialize_crt);
        if (scrt_init_map.__scrt_initialize_crt)(1) == 0 {
            panic!("Unable to initialise program!");
        }
        let result_e = (scrt_init_map._initterm_e)(__xi_a, __xi_z);
        let result = (scrt_init_map._initterm)(__xc_a, __xc_z);

        (win_main_map.win_main)(
            h_get_proc_id_dll as _,
            0,
            CString::new("UwU").unwrap().as_ptr(),
            0xA,
        );
        return;

        let args = vec![
            "program".to_string(),
            "--verbose".to_string(),
            "file.txt".to_string(),
        ];
        let array = CStringArray::new(args).unwrap();
        (win_main_map.main_func)(array.len() as i32, array.as_ptr());
    };
    result
}
