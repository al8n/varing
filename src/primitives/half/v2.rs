use core::num::NonZeroUsize;

use half_2::f16;

use crate::Varint;

impl Varint for f16 {
  const MIN_ENCODED_LEN: NonZeroUsize = u16::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: NonZeroUsize = u16::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    encoded_f16_varint_len(*self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, crate::EncodeError> {
    encode_f16_varint_to(*self, buf).map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), crate::DecodeError>
  where
    Self: Sized,
  {
    decode_f16_varint(buf).map_err(Into::into)
  }
}

/// Returns the encoded length of the value in LEB128 variable length format. The returned value will be in range of [`f16::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_f16_varint_len(value: f16) -> NonZeroUsize {
  crate::encoded_u16_varint_len(value.to_bits())
}

/// Encodes an `f16` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f16_varint(
  value: f16,
) -> crate::utils::Buffer<{ f16::MAX_ENCODED_LEN.get() + 1 }> {
  crate::encode_u16_varint(value.to_bits())
}

/// Encodes an `f16` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f16_varint_to(
  value: f16,
  buf: &mut [u8],
) -> Result<NonZeroUsize, crate::ConstEncodeError> {
  crate::encode_u16_varint_to(value.to_bits(), buf)
}

/// Decodes an `f16` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_f16_varint(buf: &[u8]) -> Result<(NonZeroUsize, f16), crate::ConstDecodeError> {
  match crate::decode_u16_varint(buf) {
    Ok((len, bits)) => Ok((len, f16::from_bits(bits))),
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of a sequence of `f16` values
#[inline]
pub const fn encoded_f16_sequence_len(sequence: &[f16]) -> usize {
  encode!(@sequence_encoded_len_impl sequence, encoded_f16_varint_len)
}

/// Encodes a sequence of `f16` to the buffer.
#[inline]
pub const fn encode_f16_sequence_to(
  sequence: &[f16],
  buf: &mut [u8],
) -> Result<usize, crate::ConstEncodeError> {
  encode!(@sequence_encode_to_impl buf, sequence, encode_f16_varint_to, encoded_f16_sequence_len)
}

#[cfg(test)]
mod tests;
