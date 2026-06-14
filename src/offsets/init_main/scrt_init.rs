use super::super::util::CallClass;

impl CallClass for STRUCT_MAP {}
#[derive(Debug)]
#[repr(C)]
pub struct STRUCT_MAP {
    pub __scrt_common_main_seh: extern "C" fn(),

    // BOOL __cdecl __scrt_initialize_crt(__scrt_module_type module_type)
    pub __scrt_initialize_crt: extern "C" fn(i32) -> i8,

    // void __cdecl __scrt_acquire_startup_lock(void)
    pub __scrt_acquire_startup_lock: extern "C" fn() -> i8,

    // int __cdecl _initterm_e(_PIFV* first, _PIFV* last)
    pub _initterm_e: extern "C" fn(*const u8, *const u8) -> i32,

    // void __cdecl _initterm(_PVFV* first, _PVFV* last)
    pub _initterm: extern "C" fn(*const u8, *const u8) -> i32,

    // void __cdecl __scrt_release_startup_lock(bool is_locked)
    pub __scrt_release_startup_lock: extern "C" fn(i8),

    // PGET_DYN_TLS_MAIN_CALLBACK __cdecl __scrt_get_dyn_tls_init_callback(void)
    pub __scrt_get_dyn_tls_init_callback: extern "C" fn() -> extern "C" fn(isize, usize),

    // bool __cdecl __scrt_is_nonwritable_in_current_image(void const* target)
    pub __scrt_is_nonwritable_in_current_image: extern "C" fn(*const u8) -> i8,

    // PGET_DYN_TLS_DTOR_CALLBACK __cdecl __scrt_get_dyn_tls_dtor_callback(void)
    pub __scrt_get_dyn_tls_dtor_callback: extern "C" fn() -> *const u8,

    // (Duplicate of above security check)
    pub __scrt_is_nonwritable_in_current_image_dupe: extern "C" fn(*const u8) -> i8,

    // int __cdecl _register_thread_local_exe_atexit_callback(_CPVFV callback)
    pub _register_thread_local_exe_atexit_callback: extern "C" fn(usize),

    // WORD __cdecl __scrt_get_show_window_mode(void)
    pub __scrt_get_show_window_mode: extern "C" fn() -> u16,

    // char* __cdecl _get_narrow_winmain_command_line(void)
    pub _get_narrow_winmain_command_line: extern "C" fn() -> *const i8,

    // int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nShowCmd)
    pub win_main: extern "C" fn(usize, usize, *const i8, i32) -> i32,

    // bool __cdecl __scrt_is_managed_app(void)
    pub __scrt_is_managed_app: extern "C" fn() -> i8,

    // void __cdecl _cexit(void)
    pub _cexit: extern "C" fn(),

    // void __cdecl __scrt_uninitialize_crt(bool terminating, bool is_managed_app)
    pub __scrt_uninitialize_crt: extern "C" fn(i8, i8),

    // (Duplicate check for .NET/CLR context)
    pub __scrt_is_managed_app_dupe: extern "C" fn() -> i8,

    // void __cdecl _c_exit(void)
    pub _c_exit: extern "C" fn(),

    // void __cdecl __scrt_fastfail(unsigned int code)
    pub __scrt_fastfail: extern "C" fn(u32) -> !,

    // (Duplicate of above fastfail)
    pub __scrt_fastfail_dupe: extern "C" fn(u32) -> !,

    // void __cdecl exit(int status)
    pub exit: extern "C" fn(i32),

    // void __cdecl _exit(int status)
    pub _exit: extern "C" fn(i32),
}

pub fn find(scrt_common_main_seh: u64) -> (STRUCT_MAP, STRUCT_MAP) {
    STRUCT_MAP::load_calls_to_struct(scrt_common_main_seh, 0x171)
}

// According to Studio v548
// 00007FF774162CB4 | 48:895C24 08             | mov qword ptr ss:[rsp+8],rbx
// 00007FF774162CB9 | 57                       | push rdi
// 00007FF774162CBA | 48:83EC 30               | sub rsp,30
// 00007FF774162CBE | B9 01000000              | mov ecx,1
// 00007FF774162CC3 | E8 00F9FFFF              | call <robloxstudiobeta.__scrt_initialize_crt>
// 00007FF774162CC8 | 84C0                     | test al,al
// 00007FF774162CCA | 0F84 30010000            | je robloxstudiobeta.7FF774162E00
// 00007FF774162CD0 | 40:32FF                  | xor dil,dil
// 00007FF774162CD3 | 40:887C24 20             | mov byte ptr ss:[rsp+20],dil
// 00007FF774162CD8 | E8 AFF8FFFF              | call <robloxstudiobeta.__scrt_acquire_startup_lock>
// 00007FF774162CDD | 8AD8                     | mov bl,al
// 00007FF774162CDF | 8B0D 7BCCA202            | mov ecx,dword ptr ds:[<__scrt_current_native_startup_state>]
// 00007FF774162CE5 | 83F9 01                  | cmp ecx,1
// 00007FF774162CE8 | 0F84 1D010000            | je robloxstudiobeta.7FF774162E0B
// 00007FF774162CEE | 85C9                     | test ecx,ecx
// 00007FF774162CF0 | 75 4A                    | jne robloxstudiobeta.7FF774162D3C
// 00007FF774162CF2 | C705 64CCA202 01000000   | mov dword ptr ds:[<__scrt_current_native_startup_state>],1
// 00007FF774162CFC | 48:8D15 25704900         | lea rdx,qword ptr ds:[<__xi_z>]
// 00007FF774162D03 | 48:8D0D F66F4900         | lea rcx,qword ptr ds:[<__xi_a>]
// 00007FF774162D0A | E8 BD750000              | call <robloxstudiobeta._initterm_e>
// 00007FF774162D0F | 85C0                     | test eax,eax
// 00007FF774162D11 | 74 0A                    | je robloxstudiobeta.7FF774162D1D
// 00007FF774162D13 | B8 FF000000              | mov eax,FF
// 00007FF774162D18 | E9 D8000000              | jmp robloxstudiobeta.7FF774162DF5
// 00007FF774162D1D | 48:8D15 B46F4900         | lea rdx,qword ptr ds:[<__xc_z>]
// 00007FF774162D24 | 48:8D0D 45E44500         | lea rcx,qword ptr ds:[<__xc_a>]
// 00007FF774162D2B | E8 96750000              | call <robloxstudiobeta._initterm>
// 00007FF774162D30 | C705 26CCA202 02000000   | mov dword ptr ds:[<__scrt_current_native_startup_state>],2
// 00007FF774162D3A | EB 08                    | jmp robloxstudiobeta.7FF774162D44
// 00007FF774162D3C | 40:B7 01                 | mov dil,1
// 00007FF774162D3F | 40:887C24 20             | mov byte ptr ss:[rsp+20],dil
// 00007FF774162D44 | 8ACB                     | mov cl,bl
// 00007FF774162D46 | E8 EDF9FFFF              | call <robloxstudiobeta.__scrt_release_startup_lock>
// 00007FF774162D4B | E8 E00D0000              | call <robloxstudiobeta.__scrt_get_dyn_tls_init_callback>
// 00007FF774162D50 | 48:8BD8                  | mov rbx,rax
// 00007FF774162D53 | 48:8338 00               | cmp qword ptr ds:[rax],0
// 00007FF774162D57 | 74 1E                    | je robloxstudiobeta.7FF774162D77
// 00007FF774162D59 | 48:8BC8                  | mov rcx,rax
// 00007FF774162D5C | E8 3FF9FFFF              | call <robloxstudiobeta.__scrt_is_nonwritable_in_current_image>
// 00007FF774162D61 | 84C0                     | test al,al
// 00007FF774162D63 | 74 12                    | je robloxstudiobeta.7FF774162D77
// 00007FF774162D65 | 45:33C0                  | xor r8d,r8d
// 00007FF774162D68 | 41:8D50 02               | lea edx,qword ptr ds:[r8+2]
// 00007FF774162D6C | 33C9                     | xor ecx,ecx
// 00007FF774162D6E | 48:8B03                  | mov rax,qword ptr ds:[rbx]
// 00007FF774162D71 | FF15 E1E34500            | call qword ptr ds:[<__guard_dispatch_icall_fptr>]
// 00007FF774162D77 | E8 BC0D0000              | call <robloxstudiobeta.__scrt_get_dyn_tls_dtor_callback>
// 00007FF774162D7C | 48:8BD8                  | mov rbx,rax
// 00007FF774162D7F | 48:8338 00               | cmp qword ptr ds:[rax],0
// 00007FF774162D83 | 74 14                    | je robloxstudiobeta.7FF774162D99
// 00007FF774162D85 | 48:8BC8                  | mov rcx,rax
// 00007FF774162D88 | E8 13F9FFFF              | call <robloxstudiobeta.__scrt_is_nonwritable_in_current_image>
// 00007FF774162D8D | 84C0                     | test al,al
// 00007FF774162D8F | 74 08                    | je robloxstudiobeta.7FF774162D99
// 00007FF774162D91 | 48:8B0B                  | mov rcx,qword ptr ds:[rbx]
// 00007FF774162D94 | E8 45750000              | call <robloxstudiobeta._register_thread_local_exe_atexit_callback>
// 00007FF774162D99 | E8 BA0B0000              | call <robloxstudiobeta.__scrt_get_show_window_mode>
// 00007FF774162D9E | 0FB7D8                   | movzx ebx,ax
// 00007FF774162DA1 | E8 1A750000              | call <robloxstudiobeta._get_narrow_winmain_command_line>
// 00007FF774162DA6 | 44:8BCB                  | mov r9d,ebx
// 00007FF774162DA9 | 4C:8BC0                  | mov r8,rax
// 00007FF774162DAC | 33D2                     | xor edx,edx
// 00007FF774162DAE | 48:8D0D 4BD20CFD         | lea rcx,qword ptr ds:[7FF771230000]
// 00007FF774162DB5 | E8 F6C12000              | call <robloxstudiobeta.WinMain>
// 00007FF774162DBA | 8BD8                     | mov ebx,eax
// 00007FF774162DBC | E8 D30B0000              | call <robloxstudiobeta.__scrt_is_managed_app>
// 00007FF774162DC1 | 84C0                     | test al,al
// 00007FF774162DC3 | 74 50                    | je robloxstudiobeta.7FF774162E15
// 00007FF774162DC5 | 40:84FF                  | test dil,dil
// 00007FF774162DC8 | 75 05                    | jne robloxstudiobeta.7FF774162DCF
// 00007FF774162DCA | E8 D9740000              | call <robloxstudiobeta._cexit>
// 00007FF774162DCF | 33D2                     | xor edx,edx
// 00007FF774162DD1 | B1 01                    | mov cl,1
// 00007FF774162DD3 | E8 84F9FFFF              | call <robloxstudiobeta.__scrt_uninitialize_crt>
// 00007FF774162DD8 | 8BC3                     | mov eax,ebx
// 00007FF774162DDA | EB 19                    | jmp robloxstudiobeta.7FF774162DF5
// 00007FF774162DDC | 8BD8                     | mov ebx,eax
// 00007FF774162DDE | E8 B10B0000              | call <robloxstudiobeta.__scrt_is_managed_app>
// 00007FF774162DE3 | 84C0                     | test al,al
// 00007FF774162DE5 | 74 36                    | je robloxstudiobeta.7FF774162E1D
// 00007FF774162DE7 | 807C24 20 00             | cmp byte ptr ss:[rsp+20],0
// 00007FF774162DEC | 75 05                    | jne robloxstudiobeta.7FF774162DF3
// 00007FF774162DEE | E8 E5740000              | call <robloxstudiobeta._c_exit>
// 00007FF774162DF3 | 8BC3                     | mov eax,ebx
// 00007FF774162DF5 | 48:8B5C24 40             | mov rbx,qword ptr ss:[rsp+40]
// 00007FF774162DFA | 48:83C4 30               | add rsp,30
// 00007FF774162DFE | 5F                       | pop rdi
// 00007FF774162DFF | C3                       | ret
// 00007FF774162E00 | B9 07000000              | mov ecx,7
// 00007FF774162E05 | E8 020A0000              | call <robloxstudiobeta.__scrt_fastfail>
// 00007FF774162E0A | 90                       | nop
// 00007FF774162E0B | B9 07000000              | mov ecx,7
// 00007FF774162E10 | E8 F7090000              | call <robloxstudiobeta.__scrt_fastfail>
// 00007FF774162E15 | 8BCB                     | mov ecx,ebx
// 00007FF774162E17 | E8 00730000              | call <robloxstudiobeta.exit>
// 00007FF774162E1C | 90                       | nop
// 00007FF774162E1D | 8BCB                     | mov ecx,ebx
// 00007FF774162E1F | E8 62720000              | call <robloxstudiobeta._exit>
// 00007FF774162E24 | 90                       | nop
