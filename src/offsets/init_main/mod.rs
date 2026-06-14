mod scrt_init;
mod win_main;
pub struct InitMain {
    scrt_init_map: scrt_init::STRUCT_MAP,
    scrt_init_callers: scrt_init::STRUCT_MAP,

    win_main_map: win_main::STRUCT_MAP,
    win_main_callers: win_main::STRUCT_MAP,

    __xi_a: *const u8,
    __xi_z: *const u8,
    __xc_a: *const u8,
    __xc_z: *const u8,
}

use super::util::load_decoder;
use crate::get_func_addr;
use cstring_array::CStringArray;

pub fn find(rōblox_dll_ptr: u64, dll_main_rva: u64) -> InitMain {
    let mut result: InitMain = unsafe { std::mem::zeroed() };
    let dll_main_ptr = rōblox_dll_ptr + dll_main_rva;

    // According to Studio v548
    // 00007FF774162E28 | 48:83EC 28               | sub rsp,28
    // 00007FF774162E2C | E8 230C0000              | call <robloxstudiobeta.__security_init_cookie>
    // 00007FF774162E31 | 48:83C4 28               | add rsp,28
    // 00007FF774162E35 | E9 7AFEFFFF              | jmp <robloxstudiobeta.__scrt_common_main_seh>

    (result.scrt_init_map, result.scrt_init_callers) = {
        // This leads to the *head* of `__scrt_common_main_seh`.
        let mut decoder = load_decoder(dll_main_ptr + 0x0D, 0x05);
        let jmp_instruction = decoder.decode();
        let scrt_common_main_seh = jmp_instruction.near_branch_target();
        scrt_init::find(scrt_common_main_seh)
    };

    (result.win_main_map, result.win_main_callers) = {
        let win_main_addr = get_func_addr!(result.scrt_init_map.win_main);
        win_main::find(win_main_addr)
    };

    // Gets the addresses of the two `lea` instructions prior to the call to `_initterm_e`.
    (result.__xi_z, result.__xi_a) = {
        let ins_size = 0x7;
        let read_size_before = ins_size * 2;
        let mut decoder = load_decoder(
            get_func_addr!(result.scrt_init_callers._initterm_e) - read_size_before,
            read_size_before,
        );
        (
            decoder.decode().memory_displacement64() as _, // __xi_z
            decoder.decode().memory_displacement64() as _, // __xi_a
        )
    };

    // Gets the addresses of the two `lea` instructions prior to the call to `_initterm`.
    (result.__xc_z, result.__xc_a) = {
        let ins_size = 0x7;
        let read_size_before = ins_size * 2;
        let mut decoder = load_decoder(
            get_func_addr!(result.scrt_init_callers._initterm) - read_size_before,
            read_size_before,
        );
        (
            decoder.decode().memory_displacement64() as _, // __xi_z
            decoder.decode().memory_displacement64() as _, // __xi_a
        )
    };

    result
}

impl InitMain {
    pub fn prep_rōblox(&self) {
        println!("{:?}", self.scrt_init_map.__scrt_initialize_crt);
        if (self.scrt_init_map.__scrt_initialize_crt)(1) == 0 {
            // Makes sense to panic, since the actual `main` function may crash the entire program anyway.
            panic!("Unable to initialise program!");
        }

        // I believe that both initialisation functions *must* be called.
        let result_e = (self.scrt_init_map._initterm_e)(self.__xi_a, self.__xi_z);
        let result = (self.scrt_init_map._initterm)(self.__xc_a, self.__xc_z);
    }

    pub fn execute(self, args: Vec<String>) {
        let args_cstr = CStringArray::new(args).unwrap();
        let argc = args_cstr.len() as _;
        let argv = args_cstr.as_ptr() as _;
        (self.win_main_map.main_func)(argc, argv);
    }
}
