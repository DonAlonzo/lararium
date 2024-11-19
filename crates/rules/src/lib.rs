use lararium::prelude::*;

#[no_mangle]
pub extern "C" fn on_registry_write(
    _topic: *const u8,
    _topic_len: usize,
    _payload: *const u8,
    _payload_len: usize,
) {
    lararium_abi::registry::write(
        &Topic::from_str("0000/video/source"),
        "curator.lararium:42000".as_bytes(),
    );
    lararium_abi::registry::write(
        &Topic::from_str("0000/audio/source"),
        "curator.lararium:42001".as_bytes(),
    );
}
