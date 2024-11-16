use lararium::prelude::*;

#[no_mangle]
pub extern "C" fn on_registry_write(
    key: *const u8,
    key_len: usize,
    payload: *const u8,
    payload_len: usize,
) {
    let key = unsafe { std::slice::from_raw_parts(key, key_len) };
    let _payload = unsafe { std::slice::from_raw_parts(payload, payload_len) };
    let Ok(key) = std::str::from_utf8(key) else {
        return;
    };
    let key = Key::from_str(key);
    if key == Key::from_str("0000/command/play") {
        lararium_abi::registry::write(&Key::from_str("0000/status"), &[0x01]);
    }
}
