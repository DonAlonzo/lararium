use crate::prelude::*;
use lararium::prelude::*;

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

        pub fn delete(
            topic: *const u8,
            topic_len: usize,
        );
    }
}

pub fn read(topic: &Topic) -> Result<Value> {
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

pub fn write(
    topic: &Topic,
    value: &Value,
) {
    unsafe {
        let topic = topic.to_string();
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(value, &mut buffer).unwrap();
        host::write(topic.as_ptr(), topic.len(), buffer.as_ptr(), buffer.len());
    }
}

pub fn delete(topic: &Topic) {
    unsafe {
        let topic = topic.to_string();
        host::delete(topic.as_ptr(), topic.len());
    }
}
