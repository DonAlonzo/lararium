use lararium_mqtt::*;

impl Handler for crate::Gateway {
    async fn handle_connect(
        &self,
        connection: &Connection,
        connect: Connect,
    ) -> Connack {
        tracing::info!("Connect");
        Connack {
            reason_code: ConnectReasonCode::Success,
        }
    }

    async fn handle_disconnect(
        &self,
        connection: &Connection,
        disconnect: Disconnect,
    ) {
        tracing::info!("Disconnect");
    }

    async fn handle_ping(
        &self,
        connection: &Connection,
    ) {
        tracing::info!("Ping");
    }

    async fn handle_publish(
        &self,
        connection: &Connection,
        publish: Publish<'_>,
    ) -> Puback {
        tracing::info!("Publish");
        Puback {}
    }

    async fn handle_subscribe(
        &self,
        connection: &Connection,
        publish: Subscribe<'_>,
    ) -> Suback {
        tracing::info!("Subscribe");
        Suback {
            reason_codes: &[SubscribeReasonCode::GrantedQoS0],
        }
    }
}
