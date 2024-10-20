use super::pseudo_random::PseudoRandom;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::sync::watch;

const FLAG_BYTE: u8 = 0x7E;
const ESCAPE_BYTE: u8 = 0x7D;
const CANCEL_BYTE: u8 = 0x1A;

// EZSP-UART over ASH
// https://www.silabs.com/documents/public/user-guides/ug101-uart-gateway-protocol-reference.pdf
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    RST,
    RSTACK,
    ERROR {
        version: u8,
        code: ErrorCode,
    },
    DATA {
        frame_number: u8,
        ack_number: u8,
        retransmit: bool,
        payload: Vec<u8>,
    },
    ACK {
        ack_number: u8,
    },
    NACK {
        ack_number: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ResetUnknownReason,
}

fn crc_ccitt(msg: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for byte in msg.iter() {
        let mut x = ((crc >> 8) ^ (*byte as u16)) & 255;
        x ^= x >> 4;
        crc = (crc << 8) ^ (x << 12) ^ (x << 5) ^ x;
    }
    crc
}

pub trait BufExt {
    fn get_frame(&mut self) -> Option<Frame>;
    fn copy_to_unstuffed_bytes(
        &mut self,
        len: usize,
    ) -> Bytes;
}

pub trait BufMutExt {
    fn put_frame(
        &mut self,
        frame: &Frame,
    );
    fn put_stuffed_u8(
        &mut self,
        byte: u8,
    );
    fn put_stuffed_u16(
        &mut self,
        value: u16,
    );
    fn put_stuffed_slice(
        &mut self,
        slice: &[u8],
    );
}

impl<T: Buf> BufExt for T {
    fn get_frame(&mut self) -> Option<Frame> {
        loop {
            let buffer = self.chunk();
            if let Some(index) = buffer
                .iter()
                .position(|&byte| byte == FLAG_BYTE || byte == CANCEL_BYTE)
            {
                let frame = self.copy_to_unstuffed_bytes(index + 1);
                if frame[frame.len() - 1] == CANCEL_BYTE {
                    continue;
                }
                if frame[0] & 0b10000000 == 0 {
                    let frame_number = (frame[0] >> 4) & 0b111;
                    let ack_number = frame[0] & 0b111;
                    let retransmit = (frame[0] >> 3) & 0b1 == 1;
                    let payload = {
                        let mut pseudo_random = PseudoRandom::new();
                        let mut payload = frame[1..frame.len() - 3].to_vec();
                        for byte in &mut payload {
                            *byte ^= pseudo_random.next().unwrap();
                        }
                        payload
                    };
                    let crc_received =
                        ((frame[frame.len() - 3] as u16) << 8) | (frame[frame.len() - 2] as u16);
                    let crc_calculated = crc_ccitt(&frame[0..frame.len() - 3]);
                    if crc_received != crc_calculated {
                        continue;
                    }
                    break Some(Frame::DATA {
                        frame_number,
                        ack_number,
                        retransmit,
                        payload: payload.to_vec(),
                    });
                }
                if frame[0] & 0b01000000 == 0 {
                    let ack_number = frame[0] & 0b111;
                    if frame[0] & 0b00100000 == 0 {
                        break Some(Frame::ACK { ack_number });
                    } else {
                        break Some(Frame::NACK { ack_number });
                    }
                }
                match frame[0] {
                    0b11000000 => {
                        break Some(Frame::RST);
                    }
                    0b11000001 => {
                        break Some(Frame::RSTACK);
                    }
                    0b11000010 => {
                        let version = frame[1];
                        let code = match frame[2] {
                            0x01 => ErrorCode::ResetUnknownReason,
                            _ => continue,
                        };
                        break Some(Frame::ERROR { version, code });
                    }
                    _ => {
                        continue;
                    }
                }
            } else {
                break None;
            }
        }
    }

    fn copy_to_unstuffed_bytes(
        &mut self,
        len: usize,
    ) -> Bytes {
        let escapes = self.chunk().iter().filter(|&&x| x == ESCAPE_BYTE).count();
        let mut buffer = BytesMut::with_capacity(len - escapes);
        let mut i = 0;
        while i < len {
            i += 1;
            let byte = match self.get_u8() {
                ESCAPE_BYTE => {
                    i += 1;
                    self.get_u8() ^ 0b00100000
                }
                byte => byte,
            };
            buffer.put_u8(byte);
        }
        buffer.freeze()
    }
}

impl<T: BufMut> BufMutExt for T {
    fn put_frame(
        &mut self,
        frame: &Frame,
    ) {
        match frame {
            Frame::RST => {
                self.put_slice(&[0xC0, 0x38, 0xBC, FLAG_BYTE]);
            }
            Frame::RSTACK => {
                self.put_slice(&[0xC1, 0x38, 0xBC, FLAG_BYTE]);
            }
            Frame::ACK { ack_number } => {
                let control_byte = 0b10000000 | (ack_number & 0b111);
                self.put_stuffed_u8(control_byte);
                self.put_stuffed_u16(crc_ccitt(&[control_byte]));
                self.put_u8(FLAG_BYTE);
            }
            Frame::NACK { ack_number } => {
                let control_byte = 0b10100000 | (ack_number & 0b111);
                self.put_stuffed_u8(control_byte);
                self.put_stuffed_u16(crc_ccitt(&[control_byte]));
                self.put_u8(FLAG_BYTE);
            }
            Frame::ERROR { version, code } => {
                let error_code_byte = match code {
                    ErrorCode::ResetUnknownReason => 0x01,
                };
                let frame_data = &[0xE0, *version, error_code_byte];
                self.put_stuffed_slice(frame_data);
                self.put_stuffed_u16(crc_ccitt(frame_data));
                self.put_u8(FLAG_BYTE);
            }
            Frame::DATA {
                frame_number,
                ack_number,
                retransmit,
                payload,
            } => {
                let mut buffer = BytesMut::with_capacity(1 + payload.len() + 2 + 1);
                let mut pseudo_random = PseudoRandom::new();
                buffer.put_u8({
                    let mut control_byte = 0;
                    control_byte |= (frame_number & 0b111) << 4;
                    if *retransmit {
                        control_byte |= 1 << 3;
                    }
                    control_byte |= ack_number & 0b111;
                    control_byte
                });
                for byte in payload {
                    buffer.put_u8(*byte ^ pseudo_random.next().unwrap());
                }
                self.put_stuffed_slice(&buffer);
                self.put_stuffed_u16(crc_ccitt(&buffer));
                self.put_u8(FLAG_BYTE);
            }
        }
    }

    fn put_stuffed_u8(
        &mut self,
        byte: u8,
    ) {
        match byte {
            FLAG_BYTE | ESCAPE_BYTE | CANCEL_BYTE => {
                self.put_u8(ESCAPE_BYTE);
                self.put_u8(byte ^ 0b00100000)
            }
            _ => {
                self.put_u8(byte);
            }
        }
    }

    fn put_stuffed_u16(
        &mut self,
        value: u16,
    ) {
        self.put_stuffed_u8((value >> 8) as u8);
        self.put_stuffed_u8((value & 0xFF) as u8);
    }

    fn put_stuffed_slice(
        &mut self,
        slice: &[u8],
    ) {
        for &byte in slice {
            self.put_stuffed_u8(byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stuff_flag_byte() {
        let mut buffer = vec![];
        buffer.put_stuffed_u8(0x7E);
        assert_eq!(vec![0x7D, 0x5E], buffer);
    }

    #[test]
    fn test_unstuff_flag_byte() {
        let stuffed = vec![0x7D, 0x5E];
        let mut stuffed = stuffed.as_slice();
        let unstuffed = stuffed.copy_to_unstuffed_bytes(stuffed.len());
        assert_eq!(vec![0x7E], unstuffed);
    }

    #[test]
    fn test_stuff_escape_byte() {
        let mut buffer = vec![];
        buffer.put_stuffed_u8(0x7D);
        assert_eq!(vec![0x7D, 0x5D], buffer);
    }

    #[test]
    fn test_unstuff_escape_byte() {
        let stuffed = vec![0x7D, 0x5D];
        let mut stuffed = stuffed.as_slice();
        let unstuffed = stuffed.copy_to_unstuffed_bytes(stuffed.len());
        assert_eq!(vec![0x7D], unstuffed);
    }
}
