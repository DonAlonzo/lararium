use super::frame::{BufExt, BufMutExt, Frame};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use tokio::sync::watch;

#[derive(Clone)]
pub struct Ash {
    frame_number: Arc<AtomicU8>,
    ack_number: Arc<AtomicU8>,
    ready_sender: watch::Sender<bool>,
    ready_receiver: watch::Receiver<bool>,
    frame_sender_tx: flume::Sender<Frame>,
    frame_sender_rx: flume::Receiver<Frame>,
}

impl Ash {
    pub fn new() -> Self {
        let (ready_sender, ready_receiver) = watch::channel(false);
        let (frame_sender_tx, frame_sender_rx) = flume::unbounded();
        Self {
            frame_number: Arc::new(AtomicU8::new(0)),
            ack_number: Arc::new(AtomicU8::new(0)),
            ready_sender,
            ready_receiver,
            frame_sender_tx,
            frame_sender_rx,
        }
    }

    pub async fn wait_until_ready(&mut self) {
        while *self.ready_receiver.borrow() == false {
            let _ = self.ready_receiver.changed().await;
        }
    }

    pub fn reset(&mut self) {
        self.send_frame(Frame::RST);
    }

    pub fn send(
        &mut self,
        payload: &[u8],
    ) {
        let frame_number = self.frame_number.fetch_add(1, Ordering::Relaxed) % 0b1000;
        let ack_number = self.ack_number.load(Ordering::Relaxed);
        self.send_frame(Frame::DATA {
            frame_number,
            ack_number,
            retransmit: false,
            payload: payload.to_vec(),
        });
    }

    pub fn feed(
        &mut self,
        buffer: &[u8],
    ) -> usize {
        let mut buffer = BytesMut::from(buffer);
        let before = buffer.remaining();
        while let Some(frame) = buffer.get_frame() {
            self.feed_frame(frame);
        }
        before - buffer.remaining()
    }

    pub fn poll(&mut self) -> Option<Vec<u8>> {
        let frame = self.poll_frame()?;
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&frame);
        Some(buffer.freeze().to_vec())
    }

    pub async fn poll_async(&mut self) -> Vec<u8> {
        let frame = self.poll_frame_async().await;
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&frame);
        buffer.freeze().to_vec()
    }

    fn poll_frame(&mut self) -> Option<Frame> {
        match self.frame_sender_rx.try_recv() {
            Ok(frame) => Some(frame),
            Err(_) => None,
        }
    }

    async fn poll_frame_async(&mut self) -> Frame {
        self.frame_sender_rx.recv_async().await.unwrap()
    }

    fn send_frame(
        &mut self,
        frame: Frame,
    ) {
        self.frame_sender_tx.send(frame).unwrap()
    }

    fn feed_frame(
        &mut self,
        frame: Frame,
    ) {
        match frame {
            Frame::RST => {
                tracing::info!("RST");
            }
            Frame::RSTACK => {
                tracing::info!("RSTACK");
                let _ = self.ready_sender.send(true);
            }
            Frame::ERROR { version, code } => {
                println!("ERROR");
            }
            Frame::DATA {
                frame_number,
                ack_number,
                retransmit,
                payload,
            } => {
                tracing::info!(
                    "DATA({}, {}, {})",
                    frame_number,
                    ack_number,
                    if retransmit { 1 } else { 0 }
                );
                let ack_number = (frame_number + 1) % 0b1000;
                self.ack_number.store(ack_number, Ordering::Relaxed);
                self.send_frame(Frame::ACK { ack_number });
            }
            Frame::ACK { ack_number } => {
                tracing::info!("ACK({})", ack_number);
            }
            Frame::NAK { ack_number } => {
                tracing::info!("NAK({})", ack_number);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow() {
        let mut ash = Ash::new();
        ash.send(&[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(
            Some(Frame::DATA {
                frame_number: 0,
                ack_number: 0,
                retransmit: false,
                payload: vec![0x00, 0x00, 0x00, 0x00],
            }),
            ash.poll_frame()
        );
        ash.send(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(
            Some(Frame::DATA {
                frame_number: 1,
                ack_number: 0,
                retransmit: false,
                payload: vec![0x01, 0x02, 0x03, 0x04],
            }),
            ash.poll_frame()
        );
        ash.feed_frame(Frame::ACK { ack_number: 1 });
        ash.feed_frame(Frame::DATA {
            frame_number: 0,
            ack_number: 2,
            retransmit: false,
            payload: vec![0x10, 0x20, 0x30, 0x40],
        });
        assert_eq!(Some(Frame::ACK { ack_number: 1 }), ash.poll_frame());
        ash.feed_frame(Frame::DATA {
            frame_number: 1,
            ack_number: 2,
            retransmit: false,
            payload: vec![0x10, 0x20, 0x30, 0x40],
        });
        assert_eq!(Some(Frame::ACK { ack_number: 2 }), ash.poll_frame());
        ash.send(&[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(
            Some(Frame::DATA {
                frame_number: 2,
                ack_number: 2,
                retransmit: false,
                payload: vec![0x11, 0x22, 0x33, 0x44],
            }),
            ash.poll_frame()
        );
    }
}
