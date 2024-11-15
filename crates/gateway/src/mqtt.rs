use lararium_mqtt::{server::*, *};
use lararium_registry::Filter;

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
        self.registry_write(publish.topic_name, publish.payload)
            .await;
        Puback {}
    }

    async fn handle_subscribe(
        &self,
        subscribe: Subscribe<'_>,
    ) -> Suback {
        tracing::debug!("Client subscribed");
        let filter = Filter::from_str(subscribe.topic_name);
        self.registry
            .subscribe(subscribe.client_id, &filter)
            .unwrap();
        Suback {
            reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
        }
    }
}
