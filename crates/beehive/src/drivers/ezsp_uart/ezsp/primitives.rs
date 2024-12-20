use super::*;

impl Decode for bool {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(buffer.get_u8() != 0x00)
    }
}

impl Encode for bool {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        if *self {
            buffer.put_u8(0x01);
        } else {
            buffer.put_u8(0x00);
        }
    }
}

impl Decode for u8 {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(buffer.get_u8())
    }
}

impl Encode for u8 {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(*self)
    }
}

impl Decode for u16 {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(buffer.get_u16_le())
    }
}

impl Encode for u16 {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(*self)
    }
}

impl Decode for () {
    fn try_decode_from<B: Buf>(_buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(())
    }
}

impl Encode for () {
    fn encode_to<B: BufMut>(
        &self,
        _buffer: &mut B,
    ) {
    }
}

macro_rules! impl_tuple_encode_decode {
    ($($name:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($name: Encode),+> Encode for ($($name,)+) {
            fn encode_to<B: BufMut>(&self, buffer: &mut B) {
                let ($($name,)+) = self;
                $(
                    $name.encode_to(buffer);
                )+
            }
        }

        #[allow(non_snake_case)]
        impl<$($name: Decode),+> Decode for ($($name,)+) {
            fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
                Ok(($(
                    $name::try_decode_from(buffer)?,
                )+))
            }
        }
    }
}

impl_tuple_encode_decode!(T1);
impl_tuple_encode_decode!(T1, T2);
impl_tuple_encode_decode!(T1, T2, T3);
impl_tuple_encode_decode!(T1, T2, T3, T4);
impl_tuple_encode_decode!(T1, T2, T3, T4, T5);
impl_tuple_encode_decode!(T1, T2, T3, T4, T5, T6);
impl_tuple_encode_decode!(T1, T2, T3, T4, T5, T6, T7);
