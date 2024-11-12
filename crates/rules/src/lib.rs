#[no_mangle]
pub extern "C" fn on_mqtt_publish(
    topic_name: *const u8,
    topic_name_len: usize,
    payload: *const u8,
    payload_len: usize,
) {
    let topic_name = unsafe { std::slice::from_raw_parts(topic_name, topic_name_len) };
    let _payload = unsafe { std::slice::from_raw_parts(payload, payload_len) };
    let Ok(topic_name) = std::str::from_utf8(topic_name) else {
        return;
    };
    if topic_name == "device/0000/influx/main" {
        lararium_core::mqtt::publish("device/0001/command/play", &[0x00, 0x01, 0x02, 0x03]);
    }
}
