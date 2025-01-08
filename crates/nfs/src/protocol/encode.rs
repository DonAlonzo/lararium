use super::*;
use num_traits::{FromPrimitive, ToPrimitive};

use cookie_factory::{bytes::be_u32, multi::all, sequence::tuple, *};
use std::io::{Cursor, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap4(Vec<u32>);

#[inline(always)]
pub fn bitmap4<'a, 'b: 'a, W: Write + 'a>(bitmap: &'b Bitmap4) -> impl SerializeFn<W> + 'a {
    tuple((
        be_u32(bitmap.0.len() as u32),
        all(bitmap.0.iter().map(|x| be_u32(*x))),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap4() {
        let value = Bitmap4(vec![0x00010203]);
        let mut buffer = [0u8; 16];
        let cursor = Cursor::new(&mut buffer[..]);
        let cursor = gen_simple(bitmap4(&value), cursor).unwrap();
        let size = cursor.position() as usize;
        let buffer = cursor.into_inner();
        let buffer = &buffer[..size];
        assert_eq!(&buffer, &[0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x02, 0x03]);
    }
}
