use lararium::prelude::*;

mod host {
    #[link(wasm_import_module = "registry")]
    extern "C" {
        pub fn write(
            key: *const u8,
            key_len: usize,
            payload: *const u8,
            payload_len: usize,
        );
    }
}

pub fn write(
    key: &Key,
    payload: &[u8],
) {
    unsafe {
        let key = key.to_string();
        host::write(key.as_ptr(), key.len(), payload.as_ptr(), payload.len());
    }
}
