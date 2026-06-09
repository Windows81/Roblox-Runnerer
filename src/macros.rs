#[macro_export]
macro_rules! get_func_addr {
    ($f:expr) => {
        ($f as *const u8).addr() as u64
    };
}

#[macro_export]
macro_rules! into_func_addr {
    ($addr:expr, $ret:ty, ($($arg_ty:ty),*), $conv:literal) => { unsafe {
        // Define the function pointer type with the specific calling convention
        type FuncType = extern $conv fn($($arg_ty),*) -> $ret;
        let func: FuncType = std::mem::transmute($addr);
        func
    }};
}

#[macro_export]
macro_rules! getproc_offset_func {
    ($handle:expr, $c_name:expr, $ret:ty, ($($arg_ty:ty),*), $conv:literal) => { unsafe {
        let addr = GetProcAddress($handle, $c_name);
        // Define the function pointer type with the specific calling convention
        type FuncType = extern $conv fn($($arg_ty),*) -> $ret;
        let func: FuncType = std::mem::transmute(addr);
        func
    }};
}
