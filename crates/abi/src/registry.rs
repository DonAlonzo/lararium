use crate::prelude::*;
use lararium::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

mod host {
    #[link(wasm_import_module = "registry")]
    extern "C" {
        pub fn read(
            topic: *const u8,
            topic_len: usize,
            payload: *mut u8,
            payload_len: usize,
        ) -> usize;

        pub fn write(
            topic: *const u8,
            topic_len: usize,
            payload: *const u8,
            payload_len: usize,
        );
    }
}

pub fn read<T: DeserializeOwned>(topic: &Topic) -> Result<T> {
    let topic = topic.to_string();
    let mut buffer = Vec::new();
    let mut capacity = 256;
    loop {
        buffer.resize(capacity, 0);
        let bytes_read = unsafe {
            host::read(
                topic.as_ptr(),
                topic.len(),
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };
        if bytes_read < capacity {
            buffer.truncate(bytes_read);
            break;
        }
        capacity = bytes_read;
    }
    let Ok(value) = ciborium::de::from_reader(&buffer[..]) else {
        return Err(Error::InvalidData);
    };
    Ok(value)
}

pub fn write<T: Serialize>(
    topic: &Topic,
    payload: T,
) {
    unsafe {
        let topic = topic.to_string();
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(&payload, &mut buffer).unwrap();
        host::write(topic.as_ptr(), topic.len(), buffer.as_ptr(), buffer.len());
    }
}
