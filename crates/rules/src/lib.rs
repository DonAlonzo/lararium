wit_bindgen::generate!({
    world: "extension",
    path: "../station/wit",
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

        download_image("https://index.docker.io/donalonzo/kodi:latest", "/kodi");
        download_image(
            "https://index.docker.io/donalonzo/jellyfin:latest",
            "/jellyfin",
        );

        mount_shared_volume("kodi", "/home/lararium")?;

        create_container(&CreateContainerArgs {
            name: "kodi".into(),
            root_dir: "/kodi".into(),
            work_dir: "/".into(),
            command: "/usr/bin/kodi".into(),
            args: vec!["kodi".into()],
            env: vec![("PATH".into(), "/bin".into())],
            wayland: true,
            pipewire: true,
        })
        .expect("failed to create container");

        create_container(&CreateContainerArgs {
            name: "jellyfin".into(),
            root_dir: "/jellyfin".into(),
            work_dir: "/".into(),
            command: "/usr/bin/jellyfin".into(),
            args: vec!["jellyfin".into(), "--nowebclient".into()],
            env: vec![("PATH".into(), "/bin".into())],
            wayland: false,
            pipewire: false,
        })
        .expect("failed to create container");

        run_container("kodi");
        run_container("jellyfin");

        loop {
            let Some(_message) = mqtt.poll_message().expect("failed to poll message") else {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            };
        }
    }
}

export!(Extension);
