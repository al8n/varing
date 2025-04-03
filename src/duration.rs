use crate::utils::Buffer;

use super::{
  decode_u128_varint, encode_u128_varint, encode_u128_varint_to, encoded_u128_varint_len,
  DecodeError, EncodeError, Varint,
};

use core::time::Duration;

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> usize {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.as_secs() as u128) << 32) | (duration.subsec_nanos() as u128);
  encoded_u128_varint_len(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> Buffer<{ Duration::MAX_ENCODED_LEN + 1 }> {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.as_secs() as u128) << 32) | (duration.subsec_nanos() as u128);
  encode_u128_varint(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(duration: &Duration, buf: &mut [u8]) -> Result<usize, EncodeError> {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.as_secs() as u128) << 32) | (duration.subsec_nanos() as u128);
  encode_u128_varint_to(value, buf)
}

/// Decodes a `Duration` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_duration(buf: &[u8]) -> Result<(usize, Duration), DecodeError> {
  match decode_u128_varint(buf) {
    Ok((bytes_read, value)) => {
      let secs = (value >> 32) as u64; // get upper 64 bits
      let nanos = (value & 0xFFFFFFFF) as u32; // get lower 32 bits
      Ok((bytes_read, Duration::new(secs, nanos)))
    }
    Err(e) => Err(e),
  }
}

impl Varint for Duration {
  const MIN_ENCODED_LEN: usize = u128::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: usize = u128::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_duration_len(self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_duration_to(self, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_duration(buf)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use quickcheck_macros::quickcheck;

  #[quickcheck]
  fn encode_decode_duration(value: Duration) -> bool {
    let encoded = encode_duration(&value);
    if encoded.len() != encoded_duration_len(&value)
      || (encoded.len() > <Duration>::MAX_ENCODED_LEN)
    {
      return false;
    }

    if let Ok((bytes_read, decoded)) = decode_duration(&encoded) {
      value == decoded && encoded.len() == bytes_read
    } else {
      false
    }
  }

  #[quickcheck]
  fn encode_decode_duration_varint(value: Duration) -> bool {
    let mut buf = [0; <Duration>::MAX_ENCODED_LEN];
    let Ok(encoded_len) = value.encode(&mut buf) else {
      return false;
    };
    if encoded_len != value.encoded_len() || (value.encoded_len() > <Duration>::MAX_ENCODED_LEN) {
      return false;
    }

    if let Ok((bytes_read, decoded)) = <Duration>::decode(&buf) {
      value == decoded && encoded_len == bytes_read
    } else {
      false
    }
  }
}
