mod crc_ccitt;
mod frame;
mod pseudo_random;

use bytes::{Buf, BytesMut};
use frame::{BufExt, BufMutExt, Frame};
use std::collections::VecDeque;

pub struct Ash {
    frame_number: u8,
    ack_number: u8,
    ready: bool,
    outgoing: VecDeque<Frame>,
    incoming: VecDeque<Vec<u8>>,
}

impl Ash {
    pub fn new() -> Self {
        Self {
            frame_number: 0,
            ack_number: 0,
            ready: false,
            outgoing: VecDeque::new(),
            incoming: VecDeque::new(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn reset(&mut self) {
        self.send_frame(Frame::RST);
    }

    pub fn send(
        &mut self,
        payload: &[u8],
    ) {
        let frame_number = self.frame_number;
        self.frame_number = (frame_number + 1) % 0b1000;
        self.send_frame(Frame::DATA {
            frame_number,
            ack_number: self.ack_number,
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

    pub fn poll_incoming(&mut self) -> Option<Vec<u8>> {
        self.incoming.pop_front()
    }

    pub fn poll_outgoing(&mut self) -> Option<Vec<u8>> {
        let frame = self.poll_outgoing_frame()?;
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&frame);
        print!("  -> ");
        match frame {
            Frame::RST => println!("RST"),
            Frame::RSTACK => println!("RSTACK"),
            Frame::ERROR { .. } => println!("ERROR"),
            Frame::DATA {
                frame_number,
                ack_number,
                retransmit,
                payload,
            } => {
                print!(
                    "DATA({}, {}, {})",
                    frame_number,
                    ack_number,
                    if retransmit { 1 } else { 0 },
                );
                print!(" [");
                for byte in payload {
                    print!("0x{byte:02X}, ");
                }
                println!("]");
            }
            Frame::ACK { ack_number } => println!("ACK({ack_number})"),
            Frame::NAK { ack_number } => println!("NAK({ack_number})"),
        }
        Some(buffer.freeze().to_vec())
    }

    fn poll_outgoing_frame(&mut self) -> Option<Frame> {
        self.outgoing.pop_front()
    }

    fn send_frame(
        &mut self,
        frame: Frame,
    ) {
        self.outgoing.push_back(frame);
    }

    fn feed_frame(
        &mut self,
        frame: Frame,
    ) {
        print!("<-   ");
        match frame {
            Frame::RST => {
                println!("RST");
            }
            Frame::RSTACK => {
                println!("RSTACK");
                self.ready = true;
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
                print!(
                    "DATA({}, {}, {})",
                    frame_number,
                    ack_number,
                    if retransmit { 1 } else { 0 },
                );
                self.frame_number = ack_number;
                let ack_number = (frame_number + 1) % 0b1000;
                self.ack_number = ack_number;
                self.send_frame(Frame::ACK { ack_number });
                print!(" [");
                for byte in &payload {
                    print!("0x{byte:02X}, ");
                }
                println!("]");
                self.incoming.push_back(payload);
            }
            Frame::ACK { ack_number } => {
                println!("ACK({})", ack_number);
            }
            Frame::NAK { ack_number } => {
                println!("NAK({})", ack_number);
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
            ash.poll_outgoing_frame()
        );
        ash.send(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(
            Some(Frame::DATA {
                frame_number: 1,
                ack_number: 0,
                retransmit: false,
                payload: vec![0x01, 0x02, 0x03, 0x04],
            }),
            ash.poll_outgoing_frame()
        );
        ash.feed_frame(Frame::ACK { ack_number: 1 });
        ash.feed_frame(Frame::DATA {
            frame_number: 0,
            ack_number: 2,
            retransmit: false,
            payload: vec![0x10, 0x20, 0x30, 0x40],
        });
        assert_eq!(
            Some(Frame::ACK { ack_number: 1 }),
            ash.poll_outgoing_frame()
        );
        ash.feed_frame(Frame::DATA {
            frame_number: 1,
            ack_number: 2,
            retransmit: false,
            payload: vec![0x10, 0x20, 0x30, 0x40],
        });
        assert_eq!(
            Some(Frame::ACK { ack_number: 2 }),
            ash.poll_outgoing_frame()
        );
        ash.send(&[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(
            Some(Frame::DATA {
                frame_number: 2,
                ack_number: 2,
                retransmit: false,
                payload: vec![0x11, 0x22, 0x33, 0x44],
            }),
            ash.poll_outgoing_frame()
        );
        assert_eq!(None, ash.poll_outgoing_frame());
    }
}
