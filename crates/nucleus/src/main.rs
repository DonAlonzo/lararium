use clap::Parser;
use lararium_types::{Topic, UserId};
use std::time::Duration;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value = "localhost")]
    mqtt_host: String,
    #[arg(env, long, default_value_t = 1883)]
    mqtt_port: u16,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let mut mqtt_options =
        rumqttc::MqttOptions::new("lararium-nucleus", args.mqtt_host, args.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (client, mut event_loop) = rumqttc::AsyncClient::new(mqtt_options, 10);
    client
        .subscribe(Topic::Hello, rumqttc::QoS::ExactlyOnce)
        .await
        .unwrap();

    tokio::task::spawn(async move {
        loop {
            let user = UserId::new();
            client
                .publish(
                    Topic::Hello,
                    rumqttc::QoS::ExactlyOnce,
                    false,
                    serde_json::to_vec(&user).unwrap(),
                )
                .await
                .unwrap();
            std::thread::sleep(Duration::from_secs(1));
        }
    });

    loop {
        let event = event_loop.poll().await;
        match &event {
            Ok(rumqttc::Event::Incoming(rumqttc::Incoming::Publish(publish))) => {
                let payload: UserId = serde_json::from_slice(&publish.payload).unwrap();
                println!("{:?}", payload);
            }
            Err(error) => {
                panic!("Error = {error:?}");
            }
            _ => {}
        }
    }
}
