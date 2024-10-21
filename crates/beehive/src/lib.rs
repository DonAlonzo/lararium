mod adapters;

use bytes::{Buf, BytesMut};
use serialport::SerialPort;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Beehive {
    serialport: Arc<Mutex<Box<dyn SerialPort>>>,
    adapter: adapters::ezsp::Adapter,
}

impl Beehive {
    pub fn new(serialport: Box<dyn SerialPort>) -> Self {
        Self {
            serialport: Arc::new(Mutex::new(serialport)),
            adapter: adapters::ezsp::Adapter::new(),
        }
    }

    pub async fn reset(&mut self) {
        self.adapter.reset().await;
    }

    pub async fn wait_until_ready(&mut self) {
        self.adapter.wait_until_ready().await;
    }

    pub async fn send_query_version(&mut self) {
        self.adapter.send_query_version().await;
    }

    pub async fn init_network(&mut self) {
        self.adapter.init_network().await;
    }

    pub async fn set_initial_security_state(&mut self) {
        self.adapter.set_initial_security_state().await;
    }

    pub async fn form_network(&mut self) {
        self.adapter.form_network().await;
    }

    pub async fn poll(&mut self) {
        loop {
            let payload = self.adapter.poll_async().await;
            let mut serialport = self.serialport.lock().await;
            serialport.write_all(&payload).unwrap();
            serialport.flush().unwrap();
        }
    }

    pub async fn listen(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        loop {
            let mut read_buffer = [0; 256];
            let bytes_read = {
                let mut serialport = self.serialport.lock().await;
                match serialport.read(&mut read_buffer) {
                    Ok(bytes_read) => bytes_read,
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
            };
            buffer.extend_from_slice(&read_buffer[..bytes_read]);
            let bytes_read = self.adapter.feed(&buffer).await;
            buffer.advance(bytes_read);
        }
    }
}
