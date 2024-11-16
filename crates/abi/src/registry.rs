use lararium::prelude::*;

mod host {
    #[link(wasm_import_module = "registry")]
    extern "C" {
        pub fn write(
            topic: *const u8,
            topic_len: usize,
            payload: *const u8,
            payload_len: usize,
        );
    }
}

pub fn write(
    topic: &Topic,
    payload: &[u8],
) {
    unsafe {
        let topic = topic.to_string();
        host::write(topic.as_ptr(), topic.len(), payload.as_ptr(), payload.len());
    }
}
