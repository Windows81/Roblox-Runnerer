use iced_x86::{Decoder, DecoderOptions, Instruction};
use std::mem::transmute_copy;

// Way more than any of our structs need; it's just scratch space for transmute_copy.
const MAX_FUNCS: usize = 1024;

pub fn load_memory<'a>(addr: u64, size: u64) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(addr as *const u8, size as usize) }
}

pub fn load_decoder<'a>(addr: u64, size: u64) -> Decoder<'a> {
    Decoder::with_ip(
        usize::BITS,
        load_memory(addr, size),
        addr,
        DecoderOptions::NONE,
    )
}

/// For `#[repr(C)]` structs made up entirely of `extern fn` pointers.
pub trait CallClass: Sized {
    fn call<T: Fn() -> ()>(&self, module_address: usize, call: T) -> T {
        unsafe { transmute_copy(&(transmute_copy::<T, usize>(&call) + module_address)) }
    }

    unsafe fn new(data: &[usize; MAX_FUNCS]) -> Self {
        unsafe { transmute_copy(data) }
    }

    unsafe fn new_offset(data: &[usize; MAX_FUNCS], offset: isize) -> Self {
        unsafe { transmute_copy(&data.map(|v| v.strict_add_signed(offset))) }
    }

    /// Rebases every pointer by `offset` (e.g. RVA -> absolute).
    unsafe fn with_offset(&self, offset: isize) -> Self {
        unsafe { Self::new_offset(transmute_copy(self), offset) }
    }

    /// Walks the code at `addr` and fills one field per `call` instruction found.
    /// Field 0 is always `addr` itself.
    ///
    /// Returns `(targets, sites)`: what each call jumps to, and where the call
    /// instruction sits. The `sites` struct isn't really callable, only read its
    /// fields through `get_func_addr!`.
    unsafe fn load_calls_to_struct(addr: u64, size: u64) -> (Self, Self) {
        let mut decoder = load_decoder(addr, size);
        let mut targets = [0usize; MAX_FUNCS];
        let mut sites = [0usize; MAX_FUNCS];
        targets[0] = addr as _;

        let mut ins = Instruction::default();
        let count = size_of::<Self>() / size_of::<usize>();
        for i in 1..count {
            loop {
                decoder.decode_out(&mut ins);
                if ins.is_call_near() {
                    break;
                }
            }
            targets[i] = ins.near_branch_target() as _;
            sites[i] = ins.ip() as _;
        }

        unsafe { (transmute_copy(&targets), transmute_copy(&sites)) }
    }
}
