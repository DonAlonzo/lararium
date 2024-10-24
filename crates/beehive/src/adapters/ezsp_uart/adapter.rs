use super::{ash::Ash, ezsp::*};
use bytes::{Bytes, BytesMut};
use std::collections::VecDeque;
use tokio::sync::{oneshot, Mutex};

pub struct Adapter {
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

impl Adapter {
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

    pub async fn startup(&self) {
        use EzspConfigId::*;
        self.set_config(TcRejoinsUsingWellKnownKeyTimeoutS, 90)
            .await;
        self.set_config(TrustCenterAddressCacheSize, 2).await;
        self.set_config(FragmentDelayMs, 50).await;
        self.set_config(PanIdConflictReportThreshold, 2).await;
        self.set_config(ApplicationZdoFlags, 0x0003).await;
        self.set_config(IndirectTransmissionTimeout, 7680).await;
        self.set_config(EndDevicePollTimeout, 14).await;
        self.set_config(SecurityLevel, 5).await;
        self.set_config(StackProfile, 2).await;
        self.set_config(FragmentWindowSize, 1).await;

        self.set_policy_decision(
            EzspPolicyId::AppKeyRequestPolicy,
            EzspDecisionId::DenyAppKeyRequests,
        )
        .await;
        self.set_policy_decision(
            EzspPolicyId::TcKeyRequestPolicy,
            EzspDecisionId::AllowTcKeyRequestsAndSendCurrentKey,
        )
        .await;
        self.set_policy_bitmask(
            EzspPolicyId::TrustCenterPolicy,
            EzspDecisionBitmask::new(&[
                EzspDecisionBitmaskFlag::AllowUnsecuredRejoins,
                EzspDecisionBitmaskFlag::AllowJoins,
            ]),
        )
        .await;

        self.set_value(EzspValueId::EndDeviceKeepAliveSupportMode, 3)
            .await;
        self.set_value(EzspValueId::CcaThreshold, 0).await;

        self.set_concentrator(
            true,
            EmberConcentratorType::HighRamConcentrator,
            10,
            90,
            4,
            3,
            0,
        )
        .await;
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

    pub async fn init_network(&self) {
        let status: EmberStatus = self
            .send_command(FrameId::NetworkInit, EmberNetworkInitBitmask::NoOptions)
            .await;
        if status != EmberStatus::Success {
            panic!("network init failed: {status:?}");
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

    pub async fn set_initial_security_state(&self) {
        use EmberInitialSecurityBitmaskFlag::*;
        let status: EmberStatus = self
            .send_command(
                FrameId::SetInitialSecurityState,
                EmberInitialSecurityState {
                    bitmask: EmberInitialSecurityBitmask::new(&[
                        HavePreconfiguredKey,
                        TrustCenterGlobalLinkKey,
                        HaveNetworkKey,
                        RequireEncryptedKey,
                        TrustCenterUsesHashedLinkKey,
                    ]),
                    preconfigured_key: EmberKeyData::new([0; 16]),
                    network_key: EmberKeyData::new([0; 16]),
                    network_key_sequence_number: 0,
                    preconfigured_trust_center_eui64: EmberEUI64::new([0, 0, 0, 0, 0, 0, 0, 0]),
                },
            )
            .await;
        if status != EmberStatus::Success {
            panic!("set initial security state failed: {status:?}");
        }
    }

    pub async fn form_network(&self) {
        let status: EmberStatus = self
            .send_command(
                FrameId::FormNetwork,
                EmberNetworkParameters {
                    extended_pan_id: 0u64,
                    pan_id: 0,
                    radio_tx_power: 10,
                    radio_channel: 11,
                    join_method: EmberJoinMethod::UseMacAssociation,
                    nwk_manager_id: 0,
                    nwk_update_id: 0,
                    channels: 0,
                },
            )
            .await;
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

    pub async fn set_value(
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

    pub async fn permit_joining(&self) {
        let duration: u8 = 255;
        let status: EmberStatus = self.send_command(FrameId::PermitJoining, (duration,)).await;
        if status != EmberStatus::Success {
            panic!("permit joining failed: {status:?}");
        }
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
        let adapter = Adapter::new();
        assert_eq!(None, adapter.poll_outgoing().await);
    }
}
