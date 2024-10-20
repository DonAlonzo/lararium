use bytes::{Buf, BufMut, Bytes, BytesMut};
use serialport::SerialPort;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

const FLAG_BYTE: u8 = 0x7E;
const ESCAPE_BYTE: u8 = 0x7D;
const CANCEL_BYTE: u8 = 0x1A;

#[derive(Clone)]
pub struct Beehive {
    serialport: Arc<Mutex<Box<dyn SerialPort>>>,
    ready_sender: watch::Sender<bool>,
    ready_receiver: watch::Receiver<bool>,
}

// EZSP over UART
// https://www.silabs.com/documents/public/user-guides/ug101-uart-gateway-protocol-reference.pdf
enum Frame {
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
enum ErrorCode {
    ResetUnknownReason,
}

struct PseudoRandom {
    value: u8,
}

impl PseudoRandom {
    fn new() -> Self {
        PseudoRandom { value: 0x42 }
    }
}

impl Iterator for PseudoRandom {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let current_value = self.value;
        if self.value & 1 == 0 {
            self.value >>= 1;
        } else {
            self.value = (self.value >> 1) ^ 0xB8;
        }
        Some(current_value)
    }
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

trait BufExt {
    fn get_frame(&mut self) -> Option<Frame>;
    fn get_unstuffed_u8(&mut self) -> u8;
    fn copy_to_unstuffed_bytes(
        &mut self,
        len: usize,
    ) -> Bytes;
}

trait BufMutExt {
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
                if frame[index] == CANCEL_BYTE {
                    continue;
                }
                if frame[0] & 0b10000000 == 0 {
                    let frame_number = (frame[0] >> 4) & 0b111;
                    let ack_number = frame[0] & 0b111;
                    let retransmit = (frame[0] >> 3) & 0b1 == 1;
                    let payload = {
                        let mut pseudo_random = PseudoRandom::new();
                        let mut payload = frame[1..index - 2].to_vec();
                        for byte in &mut payload {
                            *byte ^= pseudo_random.next().unwrap();
                        }
                        payload
                    };
                    let crc_received = ((frame[index - 2] as u16) << 8) | (frame[index - 1] as u16);
                    let crc_calculated = crc_ccitt(&frame[0..index - 2]);
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

    fn get_unstuffed_u8(&mut self) -> u8 {
        match self.get_u8() {
            ESCAPE_BYTE => self.get_u8() ^ 0b00100000,
            byte => byte,
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
                self.put_stuffed_slice(&[0xC0, 0x38, 0xBC, FLAG_BYTE]);
            }
            Frame::RSTACK => {
                self.put_stuffed_slice(&[0xC1, 0x38, 0xBC, FLAG_BYTE]);
            }
            Frame::ACK { ack_number } => {
                let control_byte = 0b10000000 | (ack_number & 0b111);
                self.put_stuffed_u8(control_byte);
                self.put_stuffed_u16(crc_ccitt(&[control_byte]));
                self.put_stuffed_u8(FLAG_BYTE);
            }
            Frame::NACK { ack_number } => {
                let control_byte = 0b10100000 | (ack_number & 0b111);
                self.put_stuffed_u8(control_byte);
                self.put_stuffed_u16(crc_ccitt(&[control_byte]));
                self.put_stuffed_u8(FLAG_BYTE);
            }
            Frame::ERROR { version, code } => {
                let error_code_byte = match code {
                    ErrorCode::ResetUnknownReason => 0x01,
                };
                let frame_data = &[0xE0, *version, error_code_byte];
                self.put_stuffed_slice(frame_data);
                self.put_stuffed_u16(crc_ccitt(frame_data));
                self.put_stuffed_u8(FLAG_BYTE);
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
                self.put_stuffed_u8(FLAG_BYTE);
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

impl Beehive {
    pub fn new(serialport: Box<dyn SerialPort>) -> Self {
        let (ready_sender, ready_receiver) = watch::channel(false);
        Self {
            serialport: Arc::new(Mutex::new(serialport)),
            ready_sender,
            ready_receiver,
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
        let mut serialport = self.serialport.lock().await;
        serialport.write_all(&buffer).unwrap();
        serialport.flush().unwrap();
        let _ = self.ready_sender.send(false);
    }

    pub async fn send_query_version(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::DATA {
            frame_number: 0,
            ack_number: 0,
            retransmit: false,
            payload: vec![0x01, 0x00, 0x00, 0x02],
        });
        let mut serialport = self.serialport.lock().await;
        serialport.write_all(&buffer).unwrap();
        serialport.flush().unwrap();
    }

    pub async fn send_init_network(&mut self) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::DATA {
            frame_number: 1,
            ack_number: 0,
            retransmit: false,
            payload: vec![0x10],
        });
        let mut serialport = self.serialport.lock().await;
        serialport.write_all(&buffer).unwrap();
        serialport.flush().unwrap();
    }

    async fn send_ack(
        &mut self,
        ack_number: u8,
    ) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::ACK { ack_number });
        let mut serialport = self.serialport.lock().await;
        serialport.write_all(&buffer).unwrap();
        serialport.flush().unwrap();
    }

    async fn send_nack(
        &mut self,
        ack_number: u8,
    ) {
        let mut buffer = BytesMut::with_capacity(256);
        buffer.put_frame(&Frame::NACK { ack_number });
        let mut serialport = self.serialport.lock().await;
        serialport.write_all(&buffer).unwrap();
        serialport.flush().unwrap();
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
                        let _ = self.send_ack(ack_number).await;
                    }
                    Frame::ACK { ack_number } => {
                        tracing::info!("ACK {ack_number}");
                        self.send_init_network().await;
                    }
                    Frame::NACK { ack_number } => {
                        tracing::info!("NACK {ack_number}");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pseudo_random() {
        let mut random_sequence = PseudoRandom::new();
        assert_eq!(0x42, random_sequence.next().unwrap());
        assert_eq!(0x21, random_sequence.next().unwrap());
        assert_eq!(0xA8, random_sequence.next().unwrap());
        assert_eq!(0x54, random_sequence.next().unwrap());
        assert_eq!(0x2A, random_sequence.next().unwrap());
    }

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
