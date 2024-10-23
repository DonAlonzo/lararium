use super::{ash::Ash, ezsp::*};
use bytes::Bytes;
use std::collections::VecDeque;
use tokio::sync::{oneshot, Mutex};

pub struct Adapter {
    state: Mutex<State>,
    network_index: u8,
}

struct State {
    ash: Ash,
    sequence: u8,
    queue: VecDeque<Dispatch>,
}

struct Dispatch {
    //frame_id: FrameId,
    tx: oneshot::Sender<Response>,
}

impl Adapter {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State {
                ash: Ash::new(),
                sequence: 0,
                queue: VecDeque::new(),
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

    pub async fn send_query_version(
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
            state.queue.push_back(Dispatch { tx });
            rx
        };
        rx.await.unwrap();
    }

    pub async fn init_network(&self) {
        self.send_command(Command::NetworkInit(NetworkInitCommand {
            bitmask: EmberNetworkInitBitmask::NoOptions,
        }))
        .await;
    }

    pub async fn set_initial_security_state(&self) {
        self.send_command(Command::SetInitialSecurityState(
            SetInitialSecurityStateCommand {
                bitmask: EmberInitialSecurityBitmask::new(),
                preconfigured_key: EmberKeyData::new([0; 16]),
                network_key: EmberKeyData::new([0; 16]),
                network_key_sequence_number: 0,
                preconfigured_trust_center_eui64: EmberEUI64::new([0; 8]),
            },
        ))
        .await;
    }

    pub async fn form_network(&self) {
        self.send_command(Command::FormNetwork(FormNetworkCommand {
            parameters: EmberNetworkParameters {
                extended_pan_id: 0u64,
                pan_id: 0,
                radio_tx_power: 0,
                radio_channel: 11,
                join_method: EmberJoinMethod::UseMacAssociation,
                nwk_manager_id: 0,
                nwk_update_id: 0,
                channels: 0,
            },
        }))
        .await;
    }

    pub async fn get_config(&self) {
        self.send_command(Command::GetConfigurationValue(
            GetConfigurationValueCommand {
                config_id: EzspConfigId::SecurityLevel,
            },
        ))
        .await;
    }

    pub async fn callback(&self) {
        self.send_command(Command::Callback).await;
    }

    pub async fn permit_joining(&self) {
        self.send_command(Command::PermitJoining(PermitJoiningCommand {
            duration: 255,
        }))
        .await;
    }

    async fn send_command(
        &self,
        command: Command,
    ) -> Response {
        let rx = {
            let mut state = self.state.lock().await;
            let sequence = state.sequence;
            state.sequence = state.sequence.wrapping_add(1);
            let frame = FrameVersion1::Command {
                sequence,
                network_index: self.network_index,
                sleep_mode: SleepMode::Idle,
                security_enabled: false,
                padding_enabled: false,
                command,
            };
            state.ash.send(&frame.encode());
            let (tx, rx) = oneshot::channel();
            state.queue.push_back(Dispatch { tx });
            rx
        };
        rx.await.unwrap()
    }

    pub async fn feed(
        &self,
        buffer: &[u8],
    ) -> usize {
        let mut state = self.state.lock().await;
        let bytes_read = state.ash.feed(buffer);
        while let Some(response) = state.ash.poll_incoming() {
            let frame = FrameVersion1::decode(&mut Bytes::from(response.clone()));
            match frame {
                Ok(FrameVersion1::Response {
                    response,
                    callback_type,
                    ..
                }) => match callback_type {
                    CallbackType::Asynchronous => {
                        println!("{response:#?}");
                    }
                    CallbackType::Synchronous | CallbackType::None => {
                        let dispatch = state.queue.pop_front().unwrap();
                        dispatch.tx.send(response).unwrap();
                    }
                },
                _ => {
                    let frame = FrameVersion0::decode(&mut Bytes::from(response));
                    match frame {
                        Ok(FrameVersion0::Response {
                            response,
                            callback_type,
                            ..
                        }) => match callback_type {
                            CallbackType::Asynchronous => {
                                println!("{response:#?}");
                            }
                            CallbackType::Synchronous | CallbackType::None => {
                                let dispatch = state.queue.pop_front().unwrap();
                                dispatch.tx.send(response).unwrap();
                            }
                        },
                        _ => tracing::error!("invalid frame"),
                    }
                }
            };
        }
        bytes_read
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
