use lararium_mqtt::{server::*, *};
use lararium_registry::Key;

impl Handler for crate::Gateway {
    async fn handle_connect(
        &self,
        connect: Connect,
    ) -> Connack {
        self.core.read().await.handle_connect(connect).await
    }

    async fn handle_disconnect(
        &self,
        disconnect: Disconnect,
    ) {
        self.core.read().await.handle_disconnect(disconnect).await
    }

    async fn handle_ping(&self) {
        self.core.read().await.handle_ping().await
    }

    async fn handle_publish(
        &self,
        publish: Publish<'_>,
    ) -> Puback {
        self.core.read().await.handle_publish(publish).await
    }

    async fn handle_subscribe(
        &self,
        subscribe: Subscribe<'_>,
    ) -> Suback {
        self.core.read().await.handle_subscribe(subscribe).await
    }
}

impl Handler for crate::Core {
    async fn handle_connect(
        &self,
        connect: Connect,
    ) -> Connack {
        tracing::debug!("Client connected");
        Connack {
            reason_code: ConnectReasonCode::Success,
        }
    }

    async fn handle_disconnect(
        &self,
        disconnect: Disconnect,
    ) {
        tracing::debug!("Client disconnected");
    }

    async fn handle_ping(&self) {
        tracing::debug!("Client pinged");
    }

    async fn handle_publish(
        &self,
        publish: Publish<'_>,
    ) -> Puback {
        tracing::debug!("[mqtt::publish] {publish:?}");
        let key = Key::from_str(publish.topic_name);
        let subscriptions = self.registry.update(&key, publish.payload).unwrap();
        self.on_mqtt_publish(publish.topic_name.to_string(), publish.payload.to_vec())
            .await;
        Puback {}
    }

    async fn handle_subscribe(
        &self,
        subscribe: Subscribe<'_>,
    ) -> Suback {
        tracing::debug!("Client subscribed");
        Suback {
            reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
        }
    }
}
