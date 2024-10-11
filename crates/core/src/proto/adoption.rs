use crate::*;
use prost::Message;

impl Message for ProposeRequest {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
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

    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}

impl Message for ProposeResponse {
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

impl Message for AcceptRequest {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::string::encode(1, &self.certificate, buf)
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        if tag == 1 {
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
        prost::encoding::string::encoded_len(1, &self.certificate)
    }

    fn clear(&mut self) {
        self.certificate.clear();
    }
}

impl Message for AcceptResponse {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
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

    fn encoded_len(&self) -> usize {
        0
    }

    fn clear(&mut self) {}
}
