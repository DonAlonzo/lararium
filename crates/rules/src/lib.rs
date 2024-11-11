#[no_mangle]
pub extern "C" fn run() {
    lararium_core::hello(42);
}
