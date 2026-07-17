use crate::utils::Buffer;

use super::{
  ConstDecodeError, ConstEncodeError, DecodeError, EncodeError, Varint, decode_u128_varint,
  encode_u128_varint, encode_u128_varint_to, encoded_u128_varint_len,
};

use core::{num::NonZeroUsize, time::Duration};

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> NonZeroUsize {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.as_secs() as u128) << 32) | (duration.subsec_nanos() as u128);
  encoded_u128_varint_len(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(
  duration: &Duration,
) -> Buffer<{ Duration::MAX_ENCODED_LEN.get() + 1 }> {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.as_secs() as u128) << 32) | (duration.subsec_nanos() as u128);
  encode_u128_varint(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(
  duration: &Duration,
  buf: &mut [u8],
) -> Result<NonZeroUsize, ConstEncodeError> {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.as_secs() as u128) << 32) | (duration.subsec_nanos() as u128);
  encode_u128_varint_to(value, buf)
}

/// Decodes a `Duration` in LEB128 encoded format from the buffer.
///
/// Returns the bytes read and the decoded value if successful.
#[inline]
pub const fn decode_duration(buf: &[u8]) -> Result<(NonZeroUsize, Duration), ConstDecodeError> {
  match decode_u128_varint(buf) {
    Ok((bytes_read, value)) => {
      // The wire layout uses exactly 96 bits: 32 for nanos, 64 for seconds.
      // Reject any value with a bit at or above position 96 instead of silently
      // discarding those high bits (which would decode a malformed value to a
      // wrong `Duration` such as `Duration::ZERO`).
      if value >> 96 != 0 {
        return Err(ConstDecodeError::other("value out of range"));
      }
      let secs = (value >> 32) as u64; // get upper 64 bits
      let nanos = (value & 0xFFFFFFFF) as u32; // get lower 32 bits
      // A well-formed encoder always emits `subsec_nanos() < 1_000_000_000`.
      // Reject malformed input so `Duration::new`'s nanos->secs carry cannot
      // overflow `u64` and panic.
      if nanos >= 1_000_000_000 {
        return Err(ConstDecodeError::other("nanos out of range"));
      }
      Ok((bytes_read, Duration::new(secs, nanos)))
    }
    Err(e) => Err(e),
  }
}

impl Varint for Duration {
  const MIN_ENCODED_LEN: NonZeroUsize = u128::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: NonZeroUsize = u128::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    encoded_duration_len(self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    encode_duration_to(self, buf).map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_duration(buf).map_err(Into::into)
  }
}

#[cfg(test)]
mod tests;
