mod adapters;

use bytes::{Buf, BytesMut};
use serialport::SerialPort;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Beehive {
    serialport: Arc<Mutex<Box<dyn SerialPort>>>,
    adapter: Arc<adapters::ezsp_uart::Adapter>,
}

impl Beehive {
    pub fn new(serialport: Box<dyn SerialPort>) -> Self {
        Self {
            serialport: Arc::new(Mutex::new(serialport)),
            adapter: Arc::new(adapters::ezsp_uart::Adapter::new()),
        }
    }

    pub async fn reset(&mut self) {
        self.adapter.reset().await;
    }

    pub async fn wait_until_ready(&mut self) {
        // TODO replace busy wait
        loop {
            if self.adapter.is_ready().await {
                break;
            }
        }
    }

    pub async fn query_version(&mut self) {
        self.adapter.query_version(13).await;
    }

    pub async fn update_config(&mut self) {
        self.adapter.update_config().await;
    }

    pub async fn update_policy(&mut self) {
        self.adapter.update_policy().await;
    }

    pub async fn init_network(&mut self) {
        self.adapter.init_network().await;
    }

    pub async fn clear_transient_link_keys(&mut self) {
        self.adapter.clear_transient_link_keys().await;
    }

    pub async fn clear_key_table(&mut self) {
        self.adapter.clear_key_table().await;
    }

    pub async fn set_initial_security_state(&mut self) {
        self.adapter.set_initial_security_state().await;
    }

    pub async fn form_network(&mut self) {
        self.adapter.form_network().await;
    }

    pub async fn permit_joining(&mut self) {
        self.adapter.permit_joining().await;
    }

    pub async fn listen(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        loop {
            let mut serialport = self.serialport.lock().await;
            {
                let bytes_read = self.adapter.feed(&buffer).await;
                buffer.advance(bytes_read);
                if let Some(payload) = self.adapter.poll_outgoing().await {
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
