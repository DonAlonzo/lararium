use crate::Subscription;
use lararium_mqtt::{server::*, *};

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
        topic_name: &str,
        payload: &[u8],
    ) -> Puback {
        self.core
            .read()
            .await
            .handle_publish(topic_name, payload)
            .await
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
        topic_name: &str,
        payload: &[u8],
    ) -> Puback {
        tracing::debug!("[mqtt::publish] {} {:?}", topic_name, payload);
        //self.registry.write(topic_name, payload).unwrap();
        self.on_mqtt_publish(topic_name.to_string(), payload.to_vec())
            .await;
        let Some(subscriptions) = self.get_subscriptions(topic_name).await else {
            tracing::debug!("No subscriptions found for topic {topic_name}");
            return Puback {};
        };
        for Subscription { tx } in subscriptions {
            if let Err(_) = tx.send_async(payload.to_vec()).await {
                tracing::error!("Failed to send message");
            }
        }
        Puback {}
    }

    async fn handle_subscribe(
        &self,
        subscribe: Subscribe<'_>,
    ) -> Suback {
        tracing::debug!("Client subscribed");
        self.add_subscription(&subscribe.topic_name, Subscription { tx: subscribe.tx })
            .await;
        Suback {
            reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
        }
    }
}
