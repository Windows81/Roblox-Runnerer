use crate::find::util::load_calls_to_struct;

#[derive(Debug)]
#[repr(C)]
pub struct STRUCT_MAP {
    // int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nShowCmd)
    pub win_main: extern "C" fn(usize, usize, *const i8, i32) -> i32,

    // void * __cdecl operator new(unsigned __int64 count)
    // Address: 00007FF74624F019
    pub op_new_0: extern "C" fn(usize) -> *const u8,

    // void * __cdecl operator new(unsigned __int64 count)
    // Address: 00007FF74624F08D
    pub op_new_1: extern "C" fn(usize) -> *const u8,

    // int __cdecl main(int argc, char** argv)
    // Address: 00007FF74624F102
    // Note: Typically wrapped by CRT entry points that pass argc/argv before calling this.
    pub main_func: extern "C" fn(i32, *const *const i8) -> i32,

    // void __cdecl operator delete[](void *ptr, unsigned __int64 count)
    // Address: 00007FF74624F11A
    pub op_delete_array_0: extern "C" fn(*const u8, usize),

    // void __cdecl operator delete[](void *ptr, unsigned __int64 count)
    // Address: 00007FF74624F12E
    pub op_delete_array_1: extern "C" fn(*const u8, usize),
}

pub fn get(win_main_addr: u64) -> (STRUCT_MAP, STRUCT_MAP) {
    load_calls_to_struct(win_main_addr, 0x19A)
}

// According to Studio v548
// 00007FF74624EFB0 | 40:53                    | push rbx
// 00007FF74624EFB2 | 57                       | push rdi
// 00007FF74624EFB3 | 41:57                    | push r15
// 00007FF74624EFB5 | 48:83EC 70               | sub rsp,70
// 00007FF74624EFB9 | 33DB                     | xor ebx,ebx
// 00007FF74624EFBB | 895C24 40                | mov dword ptr ss:[rsp+40],ebx
// 00007FF74624EFBF | FF15 3B782400            | call qword ptr ds:[<&GetCommandLineWStub>]
// 00007FF74624EFC5 | 48:8BC8                  | mov rcx,rax
// 00007FF74624EFC8 | 48:8D5424 40             | lea rdx,qword ptr ss:[rsp+40]
// 00007FF74624EFCD | FF15 FD0B2500            | call qword ptr ds:[<&CommandLineToArgvW>]
// 00007FF74624EFD3 | 48:894424 50             | mov qword ptr ss:[rsp+50],rax
// 00007FF74624EFD8 | 48:8BF8                  | mov rdi,rax
// 00007FF74624EFDB | 48:85C0                  | test rax,rax
// 00007FF74624EFDE | 75 0D                    | jne robloxstudiobeta.7FF74624EFED
// 00007FF74624EFE0 | 48:8D43 FF               | lea rax,qword ptr ds:[rbx-1]
// 00007FF74624EFE4 | 48:83C4 70               | add rsp,70
// 00007FF74624EFE8 | 41:5F                    | pop r15
// 00007FF74624EFEA | 5F                       | pop rdi
// 00007FF74624EFEB | 5B                       | pop rbx
// 00007FF74624EFEC | C3                       | ret
// 00007FF74624EFED | 8B4424 40                | mov eax,dword ptr ss:[rsp+40]
// 00007FF74624EFF1 | 49:C7C7 FFFFFFFF         | mov r15,FFFFFFFFFFFFFFFF
// 00007FF74624EFF8 | FFC0                     | inc eax
// 00007FF74624EFFA | 48:89B424 98000000       | mov qword ptr ss:[rsp+98],rsi
// 00007FF74624F002 | 48:63C8                  | movsxd rcx,eax
// 00007FF74624F005 | B8 08000000              | mov eax,8
// 00007FF74624F00A | 48:F7E1                  | mul rcx
// 00007FF74624F00D | 4C:897424 60             | mov qword ptr ss:[rsp+60],r14
// 00007FF74624F012 | 49:0F40C7                | cmovo rax,r15
// 00007FF74624F016 | 48:8BC8                  | mov rcx,rax
// 00007FF74624F019 | E8 B2EC26FF              | call <robloxstudiobeta.void * __cdecl operator new(unsigned __int64)>
// 00007FF74624F01E | 4C:8BF0                  | mov r14,rax
// 00007FF74624F021 | 48:894424 48             | mov qword ptr ss:[rsp+48],rax
// 00007FF74624F026 | 8B4424 40                | mov eax,dword ptr ss:[rsp+40]
// 00007FF74624F02A | 85C0                     | test eax,eax
// 00007FF74624F02C | 0F84 BA000000            | je robloxstudiobeta.7FF74624F0EC
// 00007FF74624F032 | 48:89AC24 90000000       | mov qword ptr ss:[rsp+90],rbp
// 00007FF74624F03A | 4C:89A424 A0000000       | mov qword ptr ss:[rsp+A0],r12
// 00007FF74624F042 | 4D:8BE6                  | mov r12,r14
// 00007FF74624F045 | 4C:896C24 68             | mov qword ptr ss:[rsp+68],r13
// 00007FF74624F04A | 4C:8BEF                  | mov r13,rdi
// 00007FF74624F04D | 4D:2BEE                  | sub r13,r14
// 00007FF74624F050 | 44:8BF3                  | mov r14d,ebx
// 00007FF74624F053 | 0F1F40 00                | nop dword ptr ds:[rax],eax
// 00007FF74624F057 | 66:0F1F8400 00000000     | nop word ptr ds:[rax+rax],ax
// 00007FF74624F060 | 4B:8B2C2C                | mov rbp,qword ptr ds:[r12+r13]
// 00007FF74624F064 | 45:8BCF                  | mov r9d,r15d
// 00007FF74624F067 | 48:895C24 38             | mov qword ptr ss:[rsp+38],rbx
// 00007FF74624F06C | 4C:8BC5                  | mov r8,rbp
// 00007FF74624F06F | 48:895C24 30             | mov qword ptr ss:[rsp+30],rbx
// 00007FF74624F074 | 33D2                     | xor edx,edx
// 00007FF74624F076 | 895C24 28                | mov dword ptr ss:[rsp+28],ebx
// 00007FF74624F07A | 33C9                     | xor ecx,ecx
// 00007FF74624F07C | 48:895C24 20             | mov qword ptr ss:[rsp+20],rbx
// 00007FF74624F081 | FF15 79762400            | call qword ptr ds:[<&WideCharToMultiByteStub>]
// 00007FF74624F087 | 48:63F8                  | movsxd rdi,eax
// 00007FF74624F08A | 48:8BCF                  | mov rcx,rdi
// 00007FF74624F08D | E8 3EEC26FF              | call <robloxstudiobeta.void * __cdecl operator new(unsigned __int64)>
// 00007FF74624F092 | 48:895C24 38             | mov qword ptr ss:[rsp+38],rbx
// 00007FF74624F097 | 45:8BCF                  | mov r9d,r15d
// 00007FF74624F09A | 48:895C24 30             | mov qword ptr ss:[rsp+30],rbx
// 00007FF74624F09F | 4C:8BC5                  | mov r8,rbp
// 00007FF74624F0A2 | 897C24 28                | mov dword ptr ss:[rsp+28],edi
// 00007FF74624F0A6 | 33D2                     | xor edx,edx
// 00007FF74624F0A8 | 33C9                     | xor ecx,ecx
// 00007FF74624F0AA | 48:894424 20             | mov qword ptr ss:[rsp+20],rax
// 00007FF74624F0AF | 48:8BF0                  | mov rsi,rax
// 00007FF74624F0B2 | FF15 48762400            | call qword ptr ds:[<&WideCharToMultiByteStub>]
// 00007FF74624F0B8 | 49:893424                | mov qword ptr ds:[r12],rsi
// 00007FF74624F0BC | 4D:8D6424 08             | lea r12,qword ptr ds:[r12+8]
// 00007FF74624F0C1 | 8B4424 40                | mov eax,dword ptr ss:[rsp+40]
// 00007FF74624F0C5 | 41:FFC6                  | inc r14d
// 00007FF74624F0C8 | 44:3BF0                  | cmp r14d,eax
// 00007FF74624F0CB | 75 93                    | jne robloxstudiobeta.7FF74624F060
// 00007FF74624F0CD | 4C:8B7424 48             | mov r14,qword ptr ss:[rsp+48]
// 00007FF74624F0D2 | 48:8B7C24 50             | mov rdi,qword ptr ss:[rsp+50]
// 00007FF74624F0D7 | 4C:8B6C24 68             | mov r13,qword ptr ss:[rsp+68]
// 00007FF74624F0DC | 4C:8BA424 A0000000       | mov r12,qword ptr ss:[rsp+A0]
// 00007FF74624F0E4 | 48:8BAC24 90000000       | mov rbp,qword ptr ss:[rsp+90]
// 00007FF74624F0EC | 48:98                    | cdqe
// 00007FF74624F0EE | 48:8BCF                  | mov rcx,rdi
// 00007FF74624F0F1 | 49:891CC6                | mov qword ptr ds:[r14+rax*8],rbx
// 00007FF74624F0F5 | FF15 F5762400            | call qword ptr ds:[<&LocalFreeStub>]
// 00007FF74624F0FB | 8B4C24 40                | mov ecx,dword ptr ss:[rsp+40]
// 00007FF74624F0FF | 49:8BD6                  | mov rdx,r14
// 00007FF74624F102 | E8 A9BA2EFD              | call <robloxstudiobeta.main>
// 00007FF74624F107 | 8BF0                     | mov esi,eax
// 00007FF74624F109 | 395C24 40                | cmp dword ptr ss:[rsp+40],ebx
// 00007FF74624F10D | 74 1C                    | je robloxstudiobeta.7FF74624F12B
// 00007FF74624F10F | 49:8BFE                  | mov rdi,r14
// 00007FF74624F112 | 48:8B0F                  | mov rcx,qword ptr ds:[rdi]
// 00007FF74624F115 | 48:85C9                  | test rcx,rcx
// 00007FF74624F118 | 74 11                    | je robloxstudiobeta.7FF74624F12B
// 00007FF74624F11A | E8 21EC26FF              | call <robloxstudiobeta.void __cdecl operator delete[](void *, unsigned __int64)>
// 00007FF74624F11F | FFC3                     | inc ebx
// 00007FF74624F121 | 48:83C7 08               | add rdi,8
// 00007FF74624F125 | 3B5C24 40                | cmp ebx,dword ptr ss:[rsp+40]
// 00007FF74624F129 | 75 E7                    | jne robloxstudiobeta.7FF74624F112
// 00007FF74624F12B | 49:8BCE                  | mov rcx,r14
// 00007FF74624F12E | E8 0DEC26FF              | call <robloxstudiobeta.void __cdecl operator delete[](void *, unsigned __int64)>
// 00007FF74624F133 | 4C:8B7424 60             | mov r14,qword ptr ss:[rsp+60]
// 00007FF74624F138 | 8BC6                     | mov eax,esi
// 00007FF74624F13A | 48:8BB424 98000000       | mov rsi,qword ptr ss:[rsp+98]
// 00007FF74624F142 | 48:83C4 70               | add rsp,70
// 00007FF74624F146 | 41:5F                    | pop r15
// 00007FF74624F148 | 5F                       | pop rdi
// 00007FF74624F149 | 5B                       | pop rbx
// 00007FF74624F14A | C3                       | ret
