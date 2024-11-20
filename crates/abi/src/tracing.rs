mod host {
    #[link(wasm_import_module = "tracing")]
    extern "C" {
        pub fn info(
            message: *const u8,
            message_len: usize,
        );
    }
}

pub fn info(message: &str) {
    unsafe {
        host::info(message.as_ptr(), message.len());
    }
}
