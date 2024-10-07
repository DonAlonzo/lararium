//use lararium_types::{Topic, UserId};
//use rumqttc::{MqttOptions, QoS};
//use std::time::Duration;

pub struct Relay {}

impl Relay {}

enum Topic {
    Hello,
}

impl Into<String> for Topic {
    fn into(self) -> String {
        match self {
            Topic::Hello => "hello".into(),
        }
    }
}

//let mut mqtt_options = MqttOptions::new("lararium-server", args.mqtt_host, args.mqtt_port);
//mqtt_options.set_keep_alive(Duration::from_secs(5));
//
//let (client, mut event_loop) = rumqttc::AsyncClient::new(mqtt_options, 10);
//client
//    .subscribe(Topic::Hello, QoS::ExactlyOnce)
//    .await?;
//
//tokio::task::spawn(async move {
//    loop {
//        let user = UserId::new();
//        client
//            .publish(
//                Topic::Hello,
//                QoS::ExactlyOnce,
//                false,
//                serde_json::to_vec(&user)?,
//            )
//            .await?;
//        std::thread::sleep(Duration::from_secs(1));
//    }
//});
//
//loop {
//    let event = event_loop.poll().await;
//    match &event {
//        Ok(rumqttc::Event::Incoming(rumqttc::Incoming::Publish(publish))) => {
//            let payload: UserId = serde_json::from_slice(&publish.payload)?;
//            println!("{:?}", payload);
//        }
//        Err(error) => {
//            panic!("Error = {error:?}");
//        }
//        _ => {}
//    }
//}
