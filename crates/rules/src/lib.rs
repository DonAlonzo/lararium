use lararium::prelude::*;
use lararium_abi::prelude::*;

#[no_mangle]
pub extern "C" fn on_registry_write(
    topic: *const u8,
    topic_len: usize,
    _payload: *const u8,
    _payload_len: usize,
) {
    let topic = unsafe { std::slice::from_raw_parts(topic, topic_len) };
    let Ok(topic) = std::str::from_utf8(topic) else {
        return;
    };
    match topic {
        "tv/power" => {
            let status_topic = Topic::from_str("tv/container/kodi/status");
            let Ok(Value::Boolean(status)) = registry::read(&status_topic) else {
                tracing::error("Failed to read status");
                return;
            };
            registry::write(&status_topic, &Value::Boolean(!status));
        }
        _ => {}
    }
}
