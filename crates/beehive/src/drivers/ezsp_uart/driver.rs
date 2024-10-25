use super::{ash::Ash, ezsp::*};
use bytes::{Bytes, BytesMut};
use std::collections::VecDeque;
use tokio::sync::{oneshot, Mutex};

pub struct Driver {
    state: Mutex<State>,
    network_index: u8,
}

struct State {
    ash: Ash,
    sequence: u8,
    protocol_version: ProtocolVersion,
    queue: VecDeque<Dispatch>,
}

struct Dispatch {
    frame_id: FrameId,
    tx: oneshot::Sender<Vec<u8>>,
}

enum ProtocolVersion {
    Version0,
    Version1,
}

impl Driver {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State {
                ash: Ash::new(),
                sequence: 0,
                queue: VecDeque::new(),
                protocol_version: ProtocolVersion::Version0,
            }),
            network_index: 0,
        }
    }

    pub async fn is_ready(&self) -> bool {
        self.state.lock().await.ash.is_ready()
    }

    pub async fn reset(&self) {
        self.state.lock().await.ash.reset();
    }

    pub async fn query_version(
        &self,
        expected_version: u8,
    ) {
        let rx = {
            let mut state = self.state.lock().await;
            let network_index = 0b00;
            let sleep_mode = SleepMode::Idle;
            let sequence = state.sequence;
            state.sequence = state.sequence.wrapping_add(1);
            let frame_control = {
                let mut byte = 0x00;
                byte |= (network_index & 0b11) << 5;
                byte | match sleep_mode {
                    SleepMode::PowerDown => 0b0000_0010,
                    SleepMode::DeepSleep => 0b0000_0001,
                    SleepMode::Idle => 0b0000_0000,
                }
            };
            state
                .ash
                .send(&[sequence, frame_control, 0x00, expected_version]);

            let (tx, rx) = oneshot::channel();
            state.queue.push_back(Dispatch {
                frame_id: FrameId::Version,
                tx,
            });
            rx
        };
        rx.await.unwrap();
        self.state.lock().await.protocol_version = ProtocolVersion::Version1;
    }

    pub async fn add_endpoint(
        &self,
        endpoint: AddEndpoint,
    ) {
        let status: EzspStatus = self.send_command(FrameId::AddEndpoint, endpoint).await;
        if status != EzspStatus::Success {
            panic!("add endpoint failed: {status:?}");
        }
    }

    pub async fn set_concentrator(
        &self,
        on: bool,
        concentrator_type: EmberConcentratorType,
        min_time: u16,
        max_time: u16,
        route_error_threshold: u8,
        delivery_failure_threshold: u8,
        max_hops: u8,
    ) {
        let status: EmberStatus = self
            .send_command(
                FrameId::SetConcentrator,
                (
                    on,
                    concentrator_type,
                    min_time,
                    max_time,
                    route_error_threshold,
                    delivery_failure_threshold,
                    max_hops,
                ),
            )
            .await;
        if status != EmberStatus::Success {
            panic!("set concentrator failed: {status:?}");
        }
    }

    pub async fn network_init(
        &self,
        bitmask: EmberNetworkInitBitmask,
    ) -> bool {
        let status: EmberStatus = self.send_command(FrameId::NetworkInit, bitmask).await;
        match status {
            EmberStatus::Success => true,
            EmberStatus::NotJoined => false,
            _ => panic!("network init failed: {status:?}"),
        }
    }

    pub async fn clear_transient_link_keys(&self) {
        let _: () = self.send_command(FrameId::ClearTransientLinkKeys, ()).await;
    }

    pub async fn clear_key_table(&self) {
        let status: EmberStatus = self.send_command(FrameId::ClearKeyTable, ()).await;
        if status != EmberStatus::Success {
            panic!("clear key table failed: {:?}", status);
        }
    }

    pub async fn set_initial_security_state(
        &self,
        initial_security_state: EmberInitialSecurityState,
    ) {
        use EmberInitialSecurityBitmaskFlag::*;
        let status: EmberStatus = self
            .send_command(FrameId::SetInitialSecurityState, initial_security_state)
            .await;
        if status != EmberStatus::Success {
            panic!("set initial security state failed: {status:?}");
        }
    }

    pub async fn form_network(
        &self,
        parameters: EmberNetworkParameters,
    ) {
        let status: EmberStatus = self.send_command(FrameId::FormNetwork, parameters).await;
        if status != EmberStatus::Success {
            panic!("form network failed: {status:?}");
        }
    }

    pub async fn get_config(
        &self,
        config_id: EzspConfigId,
    ) -> u16 {
        let (status, value): (EmberStatus, u16) = self
            .send_command(FrameId::GetConfigurationValue, config_id)
            .await;
        if status != EmberStatus::Success {
            panic!("get configuration value failed: {status:?}");
        }
        value
    }

    pub async fn set_config(
        &self,
        config_id: EzspConfigId,
        value: u16,
    ) {
        let status: EmberStatus = self
            .send_command(FrameId::SetConfigurationValue, (config_id, value))
            .await;
        if status != EmberStatus::Success {
            panic!("set configuration value failed: {status:?}");
        }
    }

    pub async fn get_policy_decision(
        &self,
        policy_id: EzspPolicyId,
    ) -> EzspDecisionId {
        let (status, value): (EzspStatus, EzspDecisionId) =
            self.send_command(FrameId::GetPolicy, policy_id).await;
        if status != EzspStatus::Success {
            panic!("get policy failed: {status:?}");
        }
        value
    }

    pub async fn set_policy_decision(
        &self,
        policy_id: EzspPolicyId,
        decision: EzspDecisionId,
    ) {
        let status: EzspStatus = self
            .send_command(FrameId::SetPolicy, (policy_id, decision))
            .await;
        if status != EzspStatus::Success {
            panic!("set policy failed: {status:?}");
        }
    }

    pub async fn set_policy_bitmask(
        &self,
        policy_id: EzspPolicyId,
        bitmask: EzspDecisionBitmask,
    ) {
        let status: EzspStatus = self
            .send_command(FrameId::SetPolicy, (policy_id, bitmask))
            .await;
        if status != EzspStatus::Success {
            panic!("set policy failed: {status:?}");
        }
    }

    pub async fn set_value_u8(
        &self,
        value_id: EzspValueId,
        value: u8,
    ) {
        let status: EzspStatus = self
            .send_command(FrameId::SetValue, (value_id, 1u8, value))
            .await;
        if status != EzspStatus::Success {
            panic!("set value failed: {status:?}");
        }
    }

    pub async fn set_value_u16(
        &self,
        value_id: EzspValueId,
        value: u16,
    ) {
        let status: EzspStatus = self
            .send_command(FrameId::SetValue, (value_id, 2u8, value))
            .await;
        if status != EzspStatus::Success {
            panic!("set value failed: {status:?}");
        }
    }

    pub async fn permit_joining(
        &self,
        duration: u8,
    ) {
        let status: EmberStatus = self.send_command(FrameId::PermitJoining, duration).await;
        if status != EmberStatus::Success {
            panic!("permit joining failed: {status:?}");
        }
    }

    pub async fn network_state(&self) -> EmberNetworkStatus {
        self.send_command(FrameId::NetworkState, ()).await
    }

    pub async fn get_network_parameters(&self) -> (EmberNodeType, EmberNetworkParameters) {
        let (status, node_type, parameters): (EmberStatus, _, _) =
            self.send_command(FrameId::GetNetworkParameters, ()).await;
        if status != EmberStatus::Success {
            panic!("get network parameters failed: {status:?}");
        }
        (node_type, parameters)
    }

    pub async fn leave_network(&self) {
        let status: EmberStatus = self.send_command(FrameId::LeaveNetwork, ()).await;
        if status != EmberStatus::Success {
            panic!("leave network failed: {status:?}");
        }
    }

    pub async fn set_manufacturer(
        &self,
        manufacturer: Manufacturer,
    ) {
        let _: () = self
            .send_command(FrameId::SetManufacturerCode, manufacturer)
            .await;
    }

    async fn send_command<E: Encode, D: Decode>(
        &self,
        frame_id: FrameId,
        parameters: E,
    ) -> D {
        let parameters = {
            let mut buffer = BytesMut::new();
            parameters.encode_to(&mut buffer);
            buffer.to_vec()
        };
        let rx = {
            let mut state = self.state.lock().await;
            let sequence = state.sequence;
            state.sequence = state.sequence.wrapping_add(1);
            let frame = FrameVersion1Command {
                sequence,
                network_index: self.network_index,
                sleep_mode: SleepMode::Idle,
                security_enabled: false,
                padding_enabled: false,
                frame_id,
                parameters,
            };
            let mut buffer = BytesMut::new();
            frame.encode_to(&mut buffer);
            state.ash.send(&buffer);
            let (tx, rx) = oneshot::channel();
            state.queue.push_back(Dispatch { frame_id, tx });
            rx
        };
        let mut response = Bytes::from(rx.await.unwrap());
        println!("{frame_id:?}");
        D::try_decode_from(&mut response).unwrap()
    }

    pub async fn feed(
        &self,
        buffer: &[u8],
    ) -> usize {
        let mut state = self.state.lock().await;
        let bytes_read = state.ash.feed(buffer);
        while let Some(response) = state.ash.poll_incoming() {
            let mut response = response.as_slice();
            match state.protocol_version {
                ProtocolVersion::Version0 => {
                    let Ok(frame) = FrameVersion0Response::try_decode_from(&mut response) else {
                        tracing::error!("invalid frame");
                        continue;
                    };
                    match frame.callback_type {
                        CallbackType::Asynchronous => {
                            self.handle_callback(frame.frame_id, frame.parameters).await;
                        }
                        CallbackType::Synchronous | CallbackType::None => {
                            let dispatch = state.queue.pop_front().unwrap();
                            dispatch.tx.send(frame.parameters).unwrap();
                        }
                    }
                }
                ProtocolVersion::Version1 => {
                    let Ok(frame) = FrameVersion1Response::try_decode_from(&mut response) else {
                        tracing::error!("invalid frame");
                        continue;
                    };
                    match frame.callback_type {
                        CallbackType::Asynchronous => {
                            self.handle_callback(frame.frame_id, frame.parameters).await;
                        }
                        CallbackType::Synchronous | CallbackType::None => {
                            let dispatch = state.queue.pop_front().unwrap();
                            dispatch.tx.send(frame.parameters).unwrap();
                        }
                    }
                }
            }
        }
        bytes_read
    }

    async fn handle_callback(
        &self,
        frame_id: FrameId,
        parameters: Vec<u8>,
    ) {
        let mut parameters = parameters.as_slice();
        match frame_id {
            FrameId::StackStatusHandler => {
                let Ok(status) = EmberStatus::try_decode_from(&mut parameters) else {
                    return;
                };
                println!("Stack status: {status:?}");
            }
            FrameId::TrustCenterJoinHandler => {
                let Ok(new_node_id) = EmberNodeId::try_decode_from(&mut parameters) else {
                    return;
                };
                let Ok(new_node_eui64) = EmberEUI64::try_decode_from(&mut parameters) else {
                    return;
                };
                let Ok(status) = EmberDeviceUpdate::try_decode_from(&mut parameters) else {
                    return;
                };
                let Ok(policy_decision) = EmberJoinDecision::try_decode_from(&mut parameters)
                else {
                    return;
                };
                let Ok(parent_of_new_node_id) = EmberNodeId::try_decode_from(&mut parameters)
                else {
                    return;
                };
                println!("Trust center join: {new_node_id}, {new_node_eui64}, {status:?}, {policy_decision:?}, {parent_of_new_node_id}");
            }
            FrameId::MessageSentHandler => {
                let Ok(message_sent) = MessageSent::try_decode_from(&mut parameters) else {
                    return;
                };
                println!(
                    "Message sent: {:?} {:02X} {:?} {:02X} {:?} {:?}",
                    message_sent.message_type,
                    message_sent.index_or_destination,
                    message_sent.aps_frame,
                    message_sent.message_tag,
                    message_sent.status,
                    message_sent.message_contents,
                );
            }
            _ => println!("callback: {frame_id:?}"),
        }
    }

    pub async fn poll_outgoing(&self) -> Option<Vec<u8>> {
        self.state.lock().await.ash.poll_outgoing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty() {
        let adapter = Driver::new();
        assert_eq!(None, adapter.poll_outgoing().await);
    }
}
