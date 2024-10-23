use super::*;

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
        Ok(buffer.get_u16())
    }
}

impl Encode for u16 {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16(*self)
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
        impl<$($name: Encode),+> Encode for ($($name,)+) {
            fn encode_to<B: BufMut>(&self, buffer: &mut B) {
                let ($($name,)+) = self;
                $(
                    $name.encode_to(buffer);
                )+
            }
        }

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
