use std::sync::LazyLock;

#[repr(C)]
pub struct CallClass {
    pub __scrt_common_main_seh: extern "C" fn(i32),
}

impl CallClass {
    pub fn call<T: Fn() -> ()>(&self, module_address: usize, call: T) -> T {
        unsafe {
            std::mem::transmute_copy(
                &(std::mem::transmute_copy::<T, usize>(&call) + module_address),
            )
        }
    }
    pub const fn new(data: &[usize; size_of::<CallClass>() / size_of::<usize>()]) -> Self {
        unsafe { std::mem::transmute(data) }
    }
}

pub static CALLS: LazyLock<CallClass> = LazyLock::new(|| CallClass::new(&[0x67000000]));
