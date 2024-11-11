mod host {
    #[link(wasm_import_module = "mqtt")]
    extern "C" {
        pub fn publish(
            topic_name: *const u8,
            topic_name_len: usize,
            payload: *const u8,
            payload_len: usize,
        );
    }
}

pub fn publish(
    topic_name: &str,
    payload: &[u8],
) {
    unsafe {
        host::publish(
            topic_name.as_ptr(),
            topic_name.len(),
            payload.as_ptr(),
            payload.len(),
        );
    }
}
