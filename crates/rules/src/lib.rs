//use lararium::prelude::*;
//use lararium_abi::prelude::*;

wit_bindgen::generate!({
    world: "host",
});

struct Host;

impl Guest for Host {
    fn run() {
        println!("Hello, world!");
        let mut stream = std::net::TcpStream::connect("127.0.0.1:1883").unwrap();
    }
}

export!(Host);
