mod scrt_init;
mod win_main;

use cstring_array::CStringArray;

use super::util::load_decoder;
use crate::get_func_addr;

use scrt_init::ScrtCalls;
use win_main::WinMainCalls;

pub struct InitMain {
    /// Call targets inside `__scrt_common_main_seh`.
    pub scrt: ScrtCalls,
    /// Addresses of the `call` instructions themselves.
    pub scrt_sites: ScrtCalls,

    pub win_main: WinMainCalls,
    pub win_main_sites: WinMainCalls,

    // CRT initializer tables. xi = C (run by _initterm_e), xc = C++ (run by _initterm).
    // _a is the start of the table and _z is the end.
    xi_a: *const u8,
    xi_z: *const u8,
    xc_a: *const u8,
    xc_z: *const u8,
}

/// Reads the two `lea rdx/rcx` right before a `call _initterm*` and returns (z, a).
fn initterm_table(call_site: u64) -> (*const u8, *const u8) {
    const LEA_LEN: u64 = 7;
    let mut decoder = load_decoder(call_site - LEA_LEN * 2, LEA_LEN * 2);
    let z = decoder.decode().memory_displacement64() as _;
    let a = decoder.decode().memory_displacement64() as _;
    (z, a)
}

pub fn find(base: u64, dll_main_rva: u64) -> InitMain {
    let dll_main = base + dll_main_rva;

    // According to Studio v548
    // 00007FF774162E28 | 48:83EC 28               | sub rsp,28
    // 00007FF774162E2C | E8 230C0000              | call <robloxstudiobeta.__security_init_cookie>
    // 00007FF774162E31 | 48:83C4 28               | add rsp,28
    // 00007FF774162E35 | E9 7AFEFFFF              | jmp <robloxstudiobeta.__scrt_common_main_seh>
    let scrt_common_main_seh = load_decoder(dll_main + 0x0D, 0x05)
        .decode()
        .near_branch_target();

    let (scrt, scrt_sites) = scrt_init::find(scrt_common_main_seh);
    let (win_main, win_main_sites) = win_main::find(get_func_addr!(scrt.win_main));

    let (xi_z, xi_a) = initterm_table(get_func_addr!(scrt_sites._initterm_e));
    let (xc_z, xc_a) = initterm_table(get_func_addr!(scrt_sites._initterm));

    InitMain {
        scrt,
        scrt_sites,
        win_main,
        win_main_sites,
        xi_a,
        xi_z,
        xc_a,
        xc_z,
    }
}

impl InitMain {
    /// Does what `__scrt_common_main_seh` would do before handing off to WinMain.
    pub fn init_crt(&self) {
        if (self.scrt.__scrt_initialize_crt)(1) == 0 {
            // Nothing past this point will work anyway, so just bail.
            panic!("Unable to initialise program!");
        }

        // I believe that both of these *must* be called, otherwise statics are left uninitialised.
        if (self.scrt._initterm_e)(self.xi_a, self.xi_z) != 0 {
            panic!("_initterm_e failed");
        }
        (self.scrt._initterm)(self.xc_a, self.xc_z);
    }

    pub fn execute(self, args: Vec<String>) -> i32 {
        let argv = CStringArray::new(args).unwrap();
        (self.win_main.main_func)(argv.len() as _, argv.as_ptr() as _)
    }
}
