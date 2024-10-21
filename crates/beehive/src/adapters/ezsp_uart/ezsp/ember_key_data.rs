use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberKeyData([u8; 16]);

impl EmberKeyData {
    pub fn new(data: [u8; 16]) -> Self {
        Self(data)
    }
}

impl Decode for EmberKeyData {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        if buffer.remaining() < 16 {
            return None;
        }
        let mut data = [0; 16];
        buffer.copy_to_slice(&mut data);
        Some(Self(data))
    }
}

impl Encode for EmberKeyData {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_slice(&self.0);
    }
}
