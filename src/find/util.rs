use std::mem::transmute_copy;

use iced_x86::{Decoder, DecoderOptions};

pub fn load_memory<'a>(addr: u64, size: u64) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(&*(addr as *const u8), size as usize) }
}

pub fn load_decoder<'a>(addr: u64, size: u64) -> Decoder<'a> {
    Decoder::with_ip(
        usize::BITS,
        load_memory(addr, size),
        addr,
        DecoderOptions::NONE,
    )
}

/// This function returns two casted C-style structs.
/// Type T is a struct whose fields' types is entirely with *extern* function pointers.
/// The first result has the *destination* addresses.
/// The second result has the *caller* instructions' addresses; values there **must** be coërced by `get_func_addr!`.
pub fn load_calls_to_struct<T: Sized>(addr: u64, size: u64) -> (T, T) {
    let mut decoder = load_decoder(addr, size);
    let mut func_list: [usize; 1024] = [0; 1024];
    let mut ptr_list: [usize; 1024] = [0; 1024];
    func_list[0] = addr as _;

    let instruction = &mut Default::default();

    let func_count = size_of::<T>() / size_of::<usize>();
    for key_index in 1..func_count {
        decoder.decode_out(instruction);
        while !instruction.is_call_near() {
            decoder.decode_out(instruction);
        }

        func_list[key_index] = instruction.near_branch_target() as _;
        ptr_list[key_index] = instruction.ip() as _;
    }
    unsafe { (transmute_copy(&func_list), transmute_copy(&ptr_list)) }
}
