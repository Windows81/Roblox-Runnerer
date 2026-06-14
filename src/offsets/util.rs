use iced_x86::{Decoder, DecoderOptions};
use std::{mem::transmute_copy, ops::Add};

const MAX_ITEM_SIZE: usize = 1024;

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

pub trait CallClass: Sized {
    fn call<T: Fn() -> ()>(&self, module_address: usize, call: T) -> T {
        unsafe { transmute_copy(&(transmute_copy::<T, usize>(&call) + module_address)) }
    }
    fn new(data: &[usize; MAX_ITEM_SIZE]) -> Self {
        unsafe { transmute_copy(data) }
    }
    fn new_offset(data: &[usize; MAX_ITEM_SIZE], offset: isize) -> Self {
        unsafe { transmute_copy(&data.map(|v| v.strict_add_signed(offset))) }
    }
    fn with_offset(&self, offset: isize) -> Self {
        Self::new_offset(unsafe { transmute_copy(self) }, offset)
    }

    /// This function returns two casted C-style structs.
    /// Type T is a struct whose fields' types is entirely with *extern* function pointers.
    /// The first result has the *destination* addresses.
    /// The second result has the *caller* instructions' addresses; values there **must** be coërced by `get_func_addr!`.
    fn load_calls_to_struct(addr: u64, size: u64) -> (Self, Self) {
        let mut decoder = load_decoder(addr, size);
        let mut func_list: [usize; MAX_ITEM_SIZE] = [0; MAX_ITEM_SIZE];
        let mut ptr_list: [usize; MAX_ITEM_SIZE] = [0; MAX_ITEM_SIZE];
        func_list[0] = addr as _;

        let instruction = &mut Default::default();

        let func_count = size_of::<Self>() / size_of::<usize>();
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
}
