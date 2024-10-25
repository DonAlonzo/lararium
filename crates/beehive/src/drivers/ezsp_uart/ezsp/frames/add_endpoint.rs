use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddEndpoint {
    pub endpoint: u8,
    pub profile_id: u16,
    pub device_id: u16,
    pub app_flags: u8,
    pub input_clusters: Vec<u16>,
    pub output_clusters: Vec<u16>,
}

impl Decode for AddEndpoint {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 8 {
            return Err(DecodeError::InsufficientData);
        }
        let endpoint = buffer.get_u8();
        let profile_id = buffer.get_u16_le();
        let device_id = buffer.get_u16_le();
        let app_flags = buffer.get_u8();
        let input_clusters_count = buffer.get_u8();
        let output_clusters_count = buffer.get_u8();
        let mut input_clusters = Vec::new();
        for _ in 0..input_clusters_count {
            input_clusters.push(buffer.get_u16_le());
        }
        let mut output_clusters = Vec::new();
        for _ in 0..output_clusters_count {
            output_clusters.push(buffer.get_u16_le());
        }
        Ok(Self {
            endpoint,
            profile_id,
            device_id,
            app_flags,
            input_clusters,
            output_clusters,
        })
    }
}

impl Encode for AddEndpoint {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(self.endpoint);
        buffer.put_u16_le(self.profile_id);
        buffer.put_u16_le(self.device_id);
        buffer.put_u8(self.app_flags);
        buffer.put_u8(self.input_clusters.len() as u8);
        buffer.put_u8(self.output_clusters.len() as u8);
        for cluster in &self.input_clusters {
            buffer.put_u16_le(*cluster);
        }
        for cluster in &self.output_clusters {
            buffer.put_u16_le(*cluster);
        }
    }
}
