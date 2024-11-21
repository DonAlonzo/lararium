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
        "0000/command/power" => {
            registry::write(
                &Topic::from_str("0000/video/source"),
                "curator.lararium:42000".as_bytes(),
            );
            registry::write(
                &Topic::from_str("0000/audio/source"),
                "curator.lararium:42001".as_bytes(),
            );
            let playing_topic = Topic::from_str("0000/power");
            let Entry::Cbor(cbor) = registry::read(&playing_topic) else {
                return;
            };
            let ciborium::Value::Bool(playing) = ciborium::de::from_reader(&cbor[..]).unwrap()
            else {
                return;
            };
            time::sleep(1000);
            registry::write(&playing_topic, &[(!playing) as u8]);
        }
        _ => {}
    }
}
