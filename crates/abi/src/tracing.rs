mod host {
    #[link(wasm_import_module = "tracing")]
    extern "C" {
        pub fn info(
            message: *const u8,
            message_len: usize,
        );
    }

    #[link(wasm_import_module = "tracing")]
    extern "C" {
        pub fn debug(
            message: *const u8,
            message_len: usize,
        );
    }

    #[link(wasm_import_module = "tracing")]
    extern "C" {
        pub fn warn(
            message: *const u8,
            message_len: usize,
        );
    }

    #[link(wasm_import_module = "tracing")]
    extern "C" {
        pub fn error(
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

pub fn debug(message: &str) {
    unsafe {
        host::debug(message.as_ptr(), message.len());
    }
}

pub fn warn(message: &str) {
    unsafe {
        host::warn(message.as_ptr(), message.len());
    }
}

pub fn error(message: &str) {
    unsafe {
        host::error(message.as_ptr(), message.len());
    }
}
