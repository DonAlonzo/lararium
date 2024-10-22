use super::{ash::Ash, ezsp::*};
use bytes::Bytes;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct Adapter {
    ash: Ash,
    sequence: AtomicU8,
    network_index: u8,
}

impl Adapter {
    pub fn new() -> Self {
        Self {
            ash: Ash::new(),
            sequence: AtomicU8::new(0),
            network_index: 0,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ash.is_ready()
    }

    pub fn reset(&self) {
        self.ash.reset();
    }

    pub async fn send_query_version(
        &self,
        expected_version: u8,
    ) {
        let network_index = 0b00;
        let sleep_mode = SleepMode::Idle;
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        let frame_control = {
            let mut byte = 0x00;
            byte |= (network_index & 0b11) << 5;
            byte | match sleep_mode {
                SleepMode::PowerDown => 0b0000_0010,
                SleepMode::DeepSleep => 0b0000_0001,
                SleepMode::Idle => 0b0000_0000,
            }
        };
        self.ash
            .send(&[sequence, frame_control, 0x00, expected_version]);
        self.ash.poll_incoming_async().await;
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
    ) -> FrameVersion1 {
        let frame = FrameVersion1::Command {
            sequence: self.sequence.fetch_add(1, Ordering::SeqCst),
            network_index: self.network_index,
            sleep_mode: SleepMode::Idle,
            security_enabled: false,
            padding_enabled: false,
            command,
        };
        self.ash.send(&frame.encode());
        let response = self.ash.poll_incoming_async().await;
        let mut response = Bytes::from(response);
        FrameVersion1::decode(&mut response)
    }

    pub fn feed(
        &self,
        buffer: &[u8],
    ) -> usize {
        self.ash.feed(buffer)
    }

    pub fn poll_outgoing(&self) -> Option<Vec<u8>> {
        self.ash.poll_outgoing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let adapter = Adapter::new();
        assert_eq!(None, adapter.poll_outgoing());
    }
}
