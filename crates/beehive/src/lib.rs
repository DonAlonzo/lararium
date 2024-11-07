mod drivers;
pub mod prelude;

use bytes::{Buf, BytesMut};
use serialport::SerialPort;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Beehive {
    serialport: Arc<Mutex<Box<dyn SerialPort>>>,
    driver: Arc<drivers::ezsp_uart::Driver>,
}

impl Beehive {
    pub fn new(serialport: Box<dyn SerialPort>) -> Self {
        Self {
            serialport: Arc::new(Mutex::new(serialport)),
            driver: Arc::new(drivers::ezsp_uart::Driver::new()),
        }
    }

    pub async fn startup(&mut self) {
        use drivers::ezsp_uart::*;

        self.driver.reset().await;
        loop {
            if self.driver.is_ready().await {
                break;
            }
        }
        self.driver.query_version(13).await;

        self.driver
            .set_config(EzspConfigId::TrustCenterAddressCacheSize, 2)
            .await;
        self.driver
            .set_config(EzspConfigId::IndirectTransmissionTimeout, 7680)
            .await;
        self.driver.set_config(EzspConfigId::MaxHops, 30).await;
        self.driver
            .set_config(EzspConfigId::SupportedNetworks, 1)
            .await;
        self.driver
            .set_policy_decision(
                EzspPolicyId::BindingModificationPolicy,
                EzspDecisionId::CheckBindingModificationsAreValidEndpointClusters,
            )
            .await;
        self.driver
            .set_policy_decision(
                EzspPolicyId::MessageContentsInCallbackPolicy,
                EzspDecisionId::MessageTagOnlyInCallback,
            )
            .await;
        self.driver
            .set_value_u16(EzspValueId::TransientDeviceTimeout, 10000)
            .await;
        self.driver.set_manufacturer(Manufacturer::Philips).await;
        self.driver.set_config(EzspConfigId::StackProfile, 2).await;
        self.driver.set_config(EzspConfigId::SecurityLevel, 5).await;
        self.driver
            .set_config(EzspConfigId::MaxEndDeviceChildren, 32)
            .await;
        self.driver
            .set_config(EzspConfigId::EndDevicePollTimeout, 8)
            .await;
        self.driver
            .set_config(EzspConfigId::TransientKeyTimeoutS, 300)
            .await;
        self.driver.set_value_u8(EzspValueId::CcaThreshold, 0).await;

        self.driver
            .add_endpoint(AddEndpoint {
                endpoint: 1,
                profile_id: 0x0104,
                device_id: 0x0005,
                app_flags: 0,
                input_clusters: vec![
                    0x0000, 0x0003, 0x0006, 0x0008, 0x0021, 0x0300, 0x0B05, 0xFC00, 0xFC01, 0xFC02,
                    0xFC03,
                ],
                output_clusters: vec![
                    0x0000, 0x0003, 0x0006, 0x0008, 0x0021, 0x0300, 0x0B05, 0xFC00, 0xFC01, 0xFC02,
                    0xFC03,
                ],
            })
            .await;

        self.driver
            .set_policy_decision(
                EzspPolicyId::TcKeyRequestPolicy,
                EzspDecisionId::AllowTcKeyRequestsAndSendCurrentKey,
            )
            .await;
        self.driver
            .set_policy_decision(
                EzspPolicyId::AppKeyRequestPolicy,
                EzspDecisionId::AllowAppKeyRequests,
            )
            .await;
        self.driver
            .set_policy_bitmask(
                EzspPolicyId::TrustCenterPolicy,
                EzspDecisionBitmask::new(&[
                    EzspDecisionBitmaskFlag::AllowUnsecuredRejoins,
                    EzspDecisionBitmaskFlag::AllowJoins,
                ]),
            )
            .await;
        let joined = self
            .driver
            .network_init(EmberNetworkInitBitmask::new(&[
                EmberNetworkInitBitmaskFlag::ParentInfoInToken,
                EmberNetworkInitBitmaskFlag::EndDeviceRejoinOnReboot,
            ]))
            .await;

        if joined {
            // wait for network up
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            //self.driver.get_network_parameters().await;
            // leave if mismatch
            self.driver.leave_network().await;
        }

        // restore if applicable
        self.driver
            .set_initial_security_state(EmberInitialSecurityState {
                bitmask: EmberInitialSecurityBitmask::new(&[
                    EmberInitialSecurityBitmaskFlag::HavePreconfiguredKey,
                    //EmberInitialSecurityBitmaskFlag::TrustCenterGlobalLinkKey,
                    EmberInitialSecurityBitmaskFlag::HaveNetworkKey,
                    //EmberInitialSecurityBitmaskFlag::RequireEncryptedKey,
                    EmberInitialSecurityBitmaskFlag::TrustCenterUsesHashedLinkKey,
                ]),
                preconfigured_key: EmberKeyData::new([
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                ]),
                network_key: EmberKeyData::new([
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                ]),
                network_key_sequence_number: 0,
                preconfigured_trust_center_eui64: EmberEUI64::new_blank(),
            })
            .await;
        self.driver.clear_key_table().await;
        //self.driver.clear_transient_link_keys().await;
        self.driver
            .form_network(EmberNetworkParameters {
                extended_pan_id: 0xDDDDDDDDDD,
                pan_id: 0x1A62,
                radio_tx_power: 5,
                radio_channel: 11,
                join_method: EmberJoinMethod::UseMacAssociation,
                nwk_manager_id: 0,
                nwk_update_id: 0,
                channels: 0x07FFF800,
            })
            .await;

        self.driver
            .set_concentrator(
                true,
                EmberConcentratorType::HighRamConcentrator,
                10,
                90,
                4,
                3,
                0,
            )
            .await;

        self.driver.permit_joining(255).await;
    }

    pub async fn listen(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        loop {
            let mut serialport = self.serialport.lock().await;
            {
                let bytes_read = self.driver.feed(&buffer).await;
                buffer.advance(bytes_read);
                if let Some(payload) = self.driver.poll_outgoing().await {
                    serialport.write_all(&payload).unwrap();
                    serialport.flush().unwrap();
                }
            }
            let mut read_buffer = [0; 256];
            match serialport.read(&mut read_buffer) {
                Ok(bytes_read) => {
                    buffer.extend_from_slice(&read_buffer[..bytes_read]);
                }
                Err(ref error) if error.kind() == io::ErrorKind::TimedOut => {
                    continue;
                }
                Err(ref error) if error.kind() == io::ErrorKind::BrokenPipe => {
                    tracing::error!("broken pipe");
                    return;
                }
                Err(error) => {
                    tracing::error!("{error}");
                    continue;
                }
            }
        }
    }
}
