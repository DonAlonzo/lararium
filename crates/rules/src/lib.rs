#![feature(wasip2)]

//use lararium::prelude::*;
//use lararium_abi::prelude::*;

wit_bindgen::generate!({
    world: "extension",
    path: "../modules/wit",
});

struct Extension;

impl Guest for Extension {
    fn run() -> Result<(), String> {
        let gateway = std::env::var("GATEWAY").expect("missing GATEWAY");
        let mqtt_port = std::env::var("MQTT_PORT")
            .expect("missing MQTT_PORT")
            .parse::<u16>()
            .expect("valid MQTT_PORT");
        let mut mqtt =
            lararium_mqtt::Client::connect(&gateway, mqtt_port).expect("failed to connect");
        mqtt.subscribe(
            lararium::Topic::from_str("hello/world"),
            lararium_mqtt::QoS::AtMostOnce,
        )
        .unwrap();

        run_container();

        loop {
            let Some(message) = mqtt.poll_message().expect("failed to poll message") else {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            };
            println!("{message:?}");
        }
    }
}

export!(Extension);
