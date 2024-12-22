//use lararium::prelude::*;
//use lararium_abi::prelude::*;

wit_bindgen::generate!({
    world: "host",
});

struct Host;

impl Guest for Host {
    fn run() {
        let gateway = std::env::var("GATEWAY").expect("missing GATEWAY");
        let mqtt_port = std::env::var("MQTT_PORT").expect("missing MQTT_PORT");
        let mut stream = std::net::TcpStream::connect((gateway, mqtt_port.parse().unwrap())).unwrap();
    }
}

export!(Host);
