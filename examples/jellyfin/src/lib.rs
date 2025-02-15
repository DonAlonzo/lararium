use mqtt::QoS;
use std::env;

wit_bindgen::generate!({
    world: "extension",
    path: "../../crates/station/wit",
});

struct Extension;

impl Guest for Extension {
    fn run() -> Result<(), String> {
        let name = env::var("NAME").expect("missing NAME");
        let _node_name = env::var("NODE_NAME").expect("missing NODE_NAME");
        let gateway = env::var("GATEWAY").expect("missing GATEWAY");
        let mqtt_port = env::var("MQTT_PORT")
            .expect("missing MQTT_PORT")
            .parse::<u16>()
            .expect("valid MQTT_PORT");

        let Ok(mut mqtt) = mqtt::Client::connect(&gateway, mqtt_port) else {
            return Err("Failed to connect to gateway".into());
        };

        mqtt.subscribe(format!("{name}/#"), QoS::AtMostOnce)
            .unwrap();

        download_image("https://index.docker.io/donalonzo/jellyfin:10.10.3-r0", "/")?;

        mount_shared_volume("/home/lararium", "jellyfin")?;

        create_container(&CreateContainerArgs {
            name: "jellyfin".into(),
            root_dir: "/".into(),
            work_dir: "/".into(),
            command: "/usr/bin/jellyfin".into(),
            args: vec!["jellyfin".into()],
            env: vec![("PATH".into(), "/bin".into())],
            wayland: true,
        })?;

        run_container("jellyfin")?;

        loop {
            let Some(_message) = mqtt.poll_message().expect("failed to poll message") else {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            };
        }
    }
}

export!(Extension);
