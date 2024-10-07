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

impl Message for Token {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::string::encode(1, &self.0, buf);
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        match tag {
            1 => prost::encoding::string::merge(wire_type, &mut self.0, buf, ctx),
            _ => prost::encoding::skip_field(wire_type, tag, buf, ctx),
        }
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        let mut len = 0;
        len += prost::encoding::string::encoded_len(1, &self.0);
        len
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl Message for LoginRequest {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::string::encode(1, &self.username, buf);
        prost::encoding::string::encode(2, &self.password, buf);
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        match tag {
            1 => prost::encoding::string::merge(wire_type, &mut self.username, buf, ctx),
            2 => prost::encoding::string::merge(wire_type, &mut self.password, buf, ctx),
            _ => prost::encoding::skip_field(wire_type, tag, buf, ctx),
        }
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        let mut len = 0;
        len += prost::encoding::string::encoded_len(1, &self.username);
        len += prost::encoding::string::encoded_len(2, &self.password);
        len
    }

    fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
    }
}

impl Message for LoginResponse {
    fn encode_raw(
        &self,
        buf: &mut impl prost::bytes::BufMut,
    ) {
        prost::encoding::message::encode(1, &self.token, buf);
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: prost::encoding::WireType,
        buf: &mut impl prost::bytes::Buf,
        ctx: prost::encoding::DecodeContext,
    ) -> Result<(), prost::DecodeError> {
        match tag {
            1 => prost::encoding::message::merge(wire_type, &mut self.token, buf, ctx).map_err(
                |mut error| {
                    error.push("", "token");
                    error
                },
            ),
            _ => prost::encoding::skip_field(wire_type, tag, buf, ctx),
        }
    }

    #[inline]
    fn encoded_len(&self) -> usize {
        let mut len = 0;
        len += prost::encoding::message::encoded_len(1, &self.token);
        len
    }

    fn clear(&mut self) {
        self.token.clear();
    }
}
