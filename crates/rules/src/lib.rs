use lararium::prelude::*;

#[no_mangle]
pub extern "C" fn on_registry_write(
    topic: *const u8,
    topic_len: usize,
    payload: *const u8,
    payload_len: usize,
) {
    let topic = unsafe { std::slice::from_raw_parts(topic, topic_len) };
    let _payload = unsafe { std::slice::from_raw_parts(payload, payload_len) };
    let Ok(topic) = std::str::from_utf8(topic) else {
        return;
    };
    let topic = Topic::from_str(topic);
    if topic == Topic::from_str("0000/command/play") {
        lararium_abi::registry::write(&Topic::from_str("0000/status"), &[0x01]);
    }
}
