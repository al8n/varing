use super::{
  decode_u32_varint, encode_u32_varint, encode_u32_varint_to, encoded_u32_varint_len, DecodeError,
  EncodeError, U32VarintBuffer, Varint,
};

/// A buffer for storing LEB128 encoded [`char`] value.
pub type CharBuffer = U32VarintBuffer;

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`char::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_char_len(char: &char) -> usize {
  encoded_u32_varint_len(*char as u32)
}

/// Encodes a `char` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_char(char: &char) -> CharBuffer {
  encode_u32_varint(*char as u32)
}

/// Encodes a `char` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_char_to(char: &char, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_u32_varint_to(*char as u32, buf)
}

/// Decodes a `char` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_char(buf: &[u8]) -> Result<(usize, char), DecodeError> {
  match decode_u32_varint(buf) {
    Ok((bytes_read, value)) => match char::from_u32(value) {
      Some(c) => Ok((bytes_read, c)),
      None => Err(DecodeError::custom("invalid char value")),
    },
    Err(e) => Err(e),
  }
}

impl Varint for char {
  const MIN_ENCODED_LEN: usize = u32::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: usize = u32::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_char_len(self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_char_to(self, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_char(buf)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use quickcheck_macros::quickcheck;

  #[quickcheck]
  fn encode_decode_char(value: char) -> bool {
    let encoded = encode_char(&value);
    if encoded.len() != encoded_char_len(&value) || (encoded.len() > <char>::MAX_ENCODED_LEN) {
      return false;
    }

    if let Ok((bytes_read, decoded)) = decode_char(&encoded) {
      value == decoded && encoded.len() == bytes_read
    } else {
      false
    }
  }

  #[quickcheck]
  fn encode_decode_char_varint(value: char) -> bool {
    let mut buf = [0; <char>::MAX_ENCODED_LEN];
    let Ok(encoded_len) = value.encode(&mut buf) else {
      return false;
    };
    if encoded_len != value.encoded_len() || (value.encoded_len() > <char>::MAX_ENCODED_LEN) {
      return false;
    }

    if let Ok((bytes_read, decoded)) = <char>::decode(&buf) {
      value == decoded && encoded_len == bytes_read
    } else {
      false
    }
  }
}
