use crate::utils::Buffer;

use super::{
  ConstDecodeError, ConstEncodeError, DecodeError, EncodeError, Varint, decode_u32_varint,
  encode_u32_varint, encode_u32_varint_to, encoded_u32_varint_len,
};

use core::num::NonZeroUsize;

/// A buffer for storing LEB128 encoded [`char`] value.
pub type CharBuffer = Buffer<{ u32::MAX_ENCODED_LEN.get() + 1 }>;

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`char::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_char_len(char: &char) -> NonZeroUsize {
  encoded_u32_varint_len(*char as u32)
}

/// Encodes a `char` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_char(char: &char) -> CharBuffer {
  encode_u32_varint(*char as u32)
}

/// Encodes a `char` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_char_to(char: &char, buf: &mut [u8]) -> Result<NonZeroUsize, ConstEncodeError> {
  encode_u32_varint_to(*char as u32, buf)
}

/// Decodes a `char` in LEB128 encoded format from the buffer.
///
/// Returns the bytes read and the decoded value if successful.
#[inline]
pub const fn decode_char(buf: &[u8]) -> Result<(NonZeroUsize, char), ConstDecodeError> {
  match decode_u32_varint(buf) {
    Ok((bytes_read, value)) => match char::from_u32(value) {
      Some(c) => Ok((bytes_read, c)),
      None => Err(ConstDecodeError::other("invalid char value")),
    },
    Err(e) => Err(e),
  }
}

impl Varint for char {
  const MIN_ENCODED_LEN: NonZeroUsize = u32::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: NonZeroUsize = u32::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    encoded_char_len(self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    encode_char_to(self, buf).map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_char(buf).map_err(Into::into)
  }
}

#[cfg(test)]
mod tests;
