use crate::*;
use prost::Message;
use uuid::Uuid;

impl Message for UserId {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::string::encode(1, &self.0.to_string(), buf)
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        if tag == 1 {
            let mut uuid_string = self.0.to_string();
            let merge_result =
                prost::encoding::string::merge(wire_type, &mut uuid_string, buf, ctx);
            self.0 = Uuid::parse_str(&uuid_string)
                .map_err(|error| prost::DecodeError::new(error.to_string()))?;
            merge_result
        } else {
            prost::encoding::skip_field(wire_type, tag, buf, ctx)
        }
    }

    fn encoded_len(&self) -> usize {
        prost::encoding::string::encoded_len(1, &self.0.to_string())
    }

    fn clear(&mut self) {
        self.0 = Uuid::nil();
    }
}

impl Message for SessionId {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::string::encode(1, &self.0.to_string(), buf)
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        if tag == 1 {
            let mut uuid_string = self.0.to_string();
            let merge_result =
                prost::encoding::string::merge(wire_type, &mut uuid_string, buf, ctx);
            self.0 = Uuid::parse_str(&uuid_string)
                .map_err(|error| prost::DecodeError::new(error.to_string()))?;
            merge_result
        } else {
            prost::encoding::skip_field(wire_type, tag, buf, ctx)
        }
    }

    fn encoded_len(&self) -> usize {
        prost::encoding::string::encoded_len(1, &self.0.to_string())
    }

    fn clear(&mut self) {
        self.0 = Uuid::nil();
    }
}

impl Message for JoinRequest {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::string::encode(1, &self.csr, buf)
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        if tag == 1 {
            let mut csr = self.csr.clone();
            let merge_result = prost::encoding::string::merge(wire_type, &mut csr, buf, ctx);
            self.csr = csr;
            merge_result
        } else {
            prost::encoding::skip_field(wire_type, tag, buf, ctx)
        }
    }

    fn encoded_len(&self) -> usize {
        prost::encoding::string::encoded_len(1, &self.csr)
    }

    fn clear(&mut self) {
        self.csr.clear();
    }
}

impl Message for JoinResponse {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::string::encode(1, &self.ca, buf);
        prost::encoding::string::encode(2, &self.certificate, buf);
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        if tag == 1 {
            let mut ca = self.ca.clone();
            let merge_result = prost::encoding::string::merge(wire_type, &mut ca, buf, ctx);
            self.ca = ca;
            merge_result
        } else if tag == 2 {
            let mut certificate = self.certificate.clone();
            let merge_result =
                prost::encoding::string::merge(wire_type, &mut certificate, buf, ctx);
            self.certificate = certificate;
            merge_result
        } else {
            prost::encoding::skip_field(wire_type, tag, buf, ctx)
        }
    }

    fn encoded_len(&self) -> usize {
        let mut len = 0;
        len += prost::encoding::string::encoded_len(1, &self.ca);
        len += prost::encoding::string::encoded_len(2, &self.certificate);
        len
    }

    fn clear(&mut self) {
        self.ca.clear();
        self.certificate.clear();
    }
}

impl Message for CheckInRequest {
    fn encode_raw(
        &self,
        _buf: &mut impl prost::bytes::BufMut,
    ) {
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        prost::encoding::skip_field(wire_type, tag, buf, ctx)
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}

impl Message for CheckInResponse {
    fn encode_raw(
        &self,
        _buf: &mut impl prost::bytes::BufMut,
    ) {
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        prost::encoding::skip_field(wire_type, tag, buf, ctx)
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}

impl Message for CheckOutRequest {
    fn encode_raw(
        &self,
        _buf: &mut impl prost::bytes::BufMut,
    ) {
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        prost::encoding::skip_field(wire_type, tag, buf, ctx)
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}

impl Message for CheckOutResponse {
    fn encode_raw(
        &self,
        _buf: &mut impl prost::bytes::BufMut,
    ) {
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        prost::encoding::skip_field(wire_type, tag, buf, ctx)
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}

impl Message for HeartbeatRequest {
    fn encode_raw(
        &self,
        _buf: &mut impl prost::bytes::BufMut,
    ) {
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        prost::encoding::skip_field(wire_type, tag, buf, ctx)
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}

impl Message for HeartbeatResponse {
    fn encode_raw(
        &self,
        _buf: &mut impl prost::bytes::BufMut,
    ) {
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        prost::encoding::skip_field(wire_type, tag, buf, ctx)
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}
