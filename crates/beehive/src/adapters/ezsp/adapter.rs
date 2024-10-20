use super::frame::{BufExt, BufMutExt, Frame};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::sync::watch;

pub struct Adapter {
    frame_number: u8,
    ack_number: u8,
    ready_sender: watch::Sender<bool>,
    ready_receiver: watch::Receiver<bool>,
    queue: [Option<Frame>; 5],
}

impl Adapter {
    pub fn new() -> Self {
        let (ready_sender, ready_receiver) = watch::channel(false);
        Self {
            frame_number: 0,
            ack_number: 0,
            ready_sender,
            ready_receiver,
            queue: [const { None::<Frame> }; 5],
        }
    }

    pub async fn wait_until_ready(&mut self) {
        while *self.ready_receiver.borrow() == false {
            let _ = self.ready_receiver.changed().await;
        }
    }

    pub async fn reset(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::RST);
    }

    pub async fn send_query_version(&mut self) {
        self.send_data(&[0x00, 0x00, 0x00, 0x02]).await;
    }

    pub async fn send_init_network(&mut self) {
        self.send_data(&[0x10]).await;
    }

    async fn send_data(
        &mut self,
        payload: &[u8],
    ) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::DATA {
            frame_number: todo!(),
            ack_number: todo!(),
            retransmit: false,
            payload: payload.to_vec(),
        });
    }

    async fn send_ack(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::ACK {
            ack_number: self.ack_number,
        });
    }

    async fn send_nack(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::ACK {
            ack_number: self.ack_number,
        });
    }

    pub async fn recv(
        &mut self,
        buffer: &[u8],
    ) -> usize {
        let before = buffer.len();
        let mut buffer = BytesMut::from(buffer);
        while let Some(frame) = buffer.get_frame() {
            match frame {
                Frame::RST => {
                    tracing::info!("RST");
                }
                Frame::RSTACK => {
                    let _ = self.ready_sender.send(true);
                }
                Frame::ERROR { version, code } => {
                    tracing::error!("ERROR {version} {code:?}");
                }
                Frame::DATA {
                    frame_number,
                    ack_number,
                    retransmit,
                    payload,
                } => {
                    tracing::info!(
                        "DATA({frame_number}, {ack_number}, {}) = {payload:?}",
                        retransmit as u8
                    );
                }
                Frame::ACK { ack_number } => {
                    tracing::info!("ACK {ack_number}");
                }
                Frame::NACK { ack_number } => {
                    tracing::info!("NACK {ack_number}");
                }
            }
        }
        buffer.len() - before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow() {}
}
