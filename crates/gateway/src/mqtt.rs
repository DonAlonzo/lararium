use lararium_mqtt::*;

impl Handler for crate::Gateway {
    async fn handle_connect(
        &self,
        connect: Connect,
    ) -> Connack {
        tracing::info!("Connect");
        Connack {
            reason_code: ConnectReasonCode::Success,
        }
    }

    async fn handle_disconnect(
        &self,
        disconnect: Disconnect,
    ) {
        tracing::info!("Disconnect");
    }

    async fn handle_ping(&self) {
        tracing::info!("Ping");
    }

    async fn handle_publish(
        &self,
        publish: Publish<'_>,
    ) -> Puback {
        tracing::info!("Publish");
        Puback {}
    }

    async fn handle_subscribe(
        &self,
        publish: Subscribe<'_>,
    ) -> Suback {
        tracing::info!("Subscribe");
        Suback {
            reason_codes: &[SubscribeReasonCode::GrantedQoS0],
        }
    }
}
