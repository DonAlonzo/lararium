use crate::Subscriber;
use lararium_mqtt::{server::*, *};

impl Handler for crate::Gateway {
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
        publish: Publish,
    ) -> Puback {
        tracing::debug!("[mqtt::publish] {publish:?}");
        match publish.payload {
            Some(payload) => {
                if let Err(error) = self.registry_write(publish.topic.clone(), payload).await {
                    tracing::error!(
                        "Error writing to entry in registry ({}): {error}",
                        publish.topic
                    );
                }
            }
            None => {
                if let Err(error) = self.registry_delete(publish.topic.clone()).await {
                    tracing::error!(
                        "Error deleting entry in registry ({}): {error}",
                        publish.topic
                    );
                }
            }
        }
        Puback {}
    }

    async fn handle_subscribe(
        &self,
        subscribe: Subscribe,
    ) -> Suback {
        tracing::debug!("Client subscribed");
        self.registry
            .subscribe(Subscriber::Client(subscribe.client_id), &subscribe.filter)
            .unwrap();
        Suback {
            reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
        }
    }
}
