mod host {
    #[link(wasm_import_module = "time")]
    extern "C" {
        pub fn sleep(milliseconds: u64);
    }
}

pub fn sleep(milliseconds: u64) {
    unsafe {
        host::sleep(milliseconds);
    }
}
