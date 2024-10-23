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

    pub async fn init_network(&self) {
        let response: NetworkInitResponse = self
            .send_command(
                FrameId::NetworkInit,
                NetworkInitCommand {
                    bitmask: EmberNetworkInitBitmask::NoOptions,
                },
            )
            .await;
        if response.status != EmberStatus::Success {
            panic!("network init failed: {:?}", response.status);
        }
    }

    pub async fn clear_transient_link_keys(&self) {
        self.send_command::<Empty>(FrameId::ClearTransientLinkKeys, Empty)
            .await;
    }

    pub async fn clear_key_table(&self) {
        let status: EmberStatus = self.send_command(FrameId::ClearKeyTable, Empty).await;
        if status != EmberStatus::Success {
            panic!("clear key table failed: {:?}", status);
        }
    }

    pub async fn set_initial_security_state(&self) {
        use EmberInitialSecurityBitmaskFlag::*;
        let response: SetInitialSecurityStateResponse = self
            .send_command(
                FrameId::SetInitialSecurityState,
                SetInitialSecurityStateCommand {
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
        if response.status != EmberStatus::Success {
            panic!("set initial security state failed: {:?}", response.status);
        }
    }

    pub async fn form_network(&self) {
        let response: FormNetworkResponse = self
            .send_command(
                FrameId::FormNetwork,
                FormNetworkCommand {
                    parameters: EmberNetworkParameters {
                        extended_pan_id: 0u64,
                        pan_id: 0,
                        radio_tx_power: 10,
                        radio_channel: 11,
                        join_method: EmberJoinMethod::UseMacAssociation,
                        nwk_manager_id: 0,
                        nwk_update_id: 0,
                        channels: 0,
                    },
                },
            )
            .await;
        if response.status != EmberStatus::Success {
            panic!("form network failed: {:?}", response.status);
        }
    }

    pub async fn get_config(&self) -> u16 {
        let response: GetConfigurationValueResponse = self
            .send_command(
                FrameId::GetConfigurationValue,
                GetConfigurationValueCommand {
                    config_id: EzspConfigId::SecurityLevel,
                },
            )
            .await;
        response.value
    }

    pub async fn permit_joining(&self) {
        let response: PermitJoiningResponse = self
            .send_command(
                FrameId::PermitJoining,
                PermitJoiningCommand { duration: 255 },
            )
            .await;
        if response.status != EmberStatus::Success {
            panic!("permit joining failed: {:?}", response.status);
        }
    }

    async fn send_command<T: Decode>(
        &self,
        frame_id: FrameId,
        parameters: impl Encode,
    ) -> T {
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
        T::try_decode_from(&mut response).unwrap()
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
                let Ok(response) = StackStatusHandlerResponse::try_decode_from(&mut parameters)
                else {
                    return;
                };
                println!("Stack status: {:#?}", response.status);
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
