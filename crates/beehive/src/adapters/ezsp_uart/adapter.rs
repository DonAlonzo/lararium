use super::{ash::Ash, ezsp::*};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct Adapter {
    ash: Ash,
    sequence: Arc<AtomicU8>,
    network_index: u8,
}

impl Adapter {
    pub fn new() -> Self {
        Self {
            ash: Ash::new(),
            sequence: Arc::new(AtomicU8::new(0)),
            network_index: 0,
        }
    }

    pub async fn reset(&mut self) {
        self.ash.reset().await;
    }

    pub async fn wait_until_ready(&mut self) {
        self.ash.wait_until_ready().await;
    }

    pub async fn send_query_version(
        &mut self,
        expected_version: u8,
    ) {
        let network_index = 0b00;
        let sleep_mode = SleepMode::Idle;
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
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
            .send(&[sequence, frame_control, 0x00, expected_version])
            .await;
    }

    pub async fn init_network(&mut self) {
        self.send_command(Command::NetworkInit(NetworkInitCommand {
            bitmask: EmberNetworkInitBitmask::NoOptions,
        }))
        .await;
    }

    pub async fn set_initial_security_state(&mut self) {
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

    pub async fn form_network(&mut self) {
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

    async fn send_command(
        &mut self,
        command: Command,
    ) {
        let frame = FrameVersion1::Command {
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            network_index: self.network_index,
            sleep_mode: SleepMode::Idle,
            security_enabled: false,
            padding_enabled: false,
            command,
        };
        self.ash.send(&frame.encode()).await;
    }

    pub async fn feed(
        &mut self,
        buffer: &[u8],
    ) -> usize {
        self.ash.feed(buffer).await
    }

    pub fn poll(&mut self) -> Option<Vec<u8>> {
        self.ash.poll()
    }

    pub async fn poll_async(&mut self) -> Vec<u8> {
        self.ash.poll_async().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let mut adapter = Adapter::new();
        assert_eq!(None, adapter.poll());
    }
}
