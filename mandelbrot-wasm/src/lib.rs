extern {
    fn log(ptr: *const u8, len: usize);
}

#[no_mangle]
pub extern fn start() {
    let s = "Hello, World!".to_owned();
    unsafe { log(s.as_ptr(), s.len()); }
}
