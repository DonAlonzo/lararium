use crate::Subscription;
use lararium_mqtt::{server::*, *};

impl Handler for crate::Gateway {
    async fn handle_connect(
        &mut self,
        connect: Connect,
    ) -> Connack {
        tracing::debug!("Client connected");
        Connack {
            reason_code: ConnectReasonCode::Success,
        }
    }

    async fn handle_disconnect(
        &mut self,
        disconnect: Disconnect,
    ) {
        tracing::debug!("Client disconnected");
    }

    async fn handle_ping(&mut self) {
        tracing::debug!("Client pinged");
    }

    async fn handle_publish(
        &mut self,
        publish: Publish<'_>,
    ) -> Puback {
        tracing::debug!("Client published");
        let Some(subscriptions) = self.get_subscriptions(publish.topic_name).await else {
            return Puback {};
        };
        for Subscription { tx } in subscriptions {
            if let Err(_) = tx.send_async(publish.payload.to_vec()).await {
                tracing::error!("Failed to send message");
            }
        }
        Puback {}
    }

    async fn handle_subscribe(
        &mut self,
        subscribe: Subscribe<'_>,
    ) -> Suback {
        tracing::debug!("Client subscribed");
        self.add_subscription(&subscribe.topic_name, Subscription { tx: subscribe.tx })
            .await;
        Suback {
            reason_codes: &[SubscribeReasonCode::GrantedQoS0],
        }
    }
}
