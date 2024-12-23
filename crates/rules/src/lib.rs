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
        let mqtt_port = std::env::var("MQTT_PORT").expect("missing MQTT_PORT");
        let mut stream =
            std::net::TcpStream::connect((gateway, mqtt_port.parse().unwrap())).unwrap();

        run_container();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(10000));
        }
    }
}

export!(Extension);
