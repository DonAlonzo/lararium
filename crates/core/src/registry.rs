mod host {
    #[link(wasm_import_module = "registry")]
    extern "C" {
        pub fn write(
            topic_name: *const u8,
            topic_name_len: usize,
            payload: *const u8,
            payload_len: usize,
        );
    }
}

pub fn write(
    topic_name: &str,
    payload: &[u8],
) {
    unsafe {
        host::write(
            topic_name.as_ptr(),
            topic_name.len(),
            payload.as_ptr(),
            payload.len(),
        );
    }
}
