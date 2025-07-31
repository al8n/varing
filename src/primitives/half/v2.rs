use half_2::f16;

use crate::Varint;

impl Varint for f16 {
  const MIN_ENCODED_LEN: usize = u16::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: usize = u16::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_f16_varint_len(*self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, crate::EncodeError> {
    encode_f16_varint_to(*self, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), crate::DecodeError>
  where
    Self: Sized,
  {
    decode_f16_varint(buf)
  }
}

/// Returns the encoded length of the value in LEB128 variable length format. The returned value will be in range of [`f16::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_f16_varint_len(value: f16) -> usize {
  crate::encoded_u16_varint_len(value.to_bits())
}

/// Encodes an `f16` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f16_varint(value: f16) -> crate::utils::Buffer<{ f16::MAX_ENCODED_LEN + 1 }> {
  crate::encode_u16_varint(value.to_bits())
}

/// Encodes an `f16` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f16_varint_to(value: f16, buf: &mut [u8]) -> Result<usize, crate::EncodeError> {
  crate::encode_u16_varint_to(value.to_bits(), buf)
}

/// Decodes an `f16` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_f16_varint(buf: &[u8]) -> Result<(usize, f16), crate::DecodeError> {
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
) -> Result<usize, crate::EncodeError> {
  encode!(@sequence_encode_to_impl buf, sequence, encode_f16_varint_to, encoded_f16_sequence_len)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, Copy)]
  struct FuzzyF16(f16);

  impl quickcheck::Arbitrary for FuzzyF16 {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
      loop {
        let val = f16::from_bits(u16::arbitrary(g));
        if !val.is_nan() {
          break Self(val);
        }
      }
    }
  }

  quickcheck::quickcheck! {
    fn fuzzy_f16_varint(value: FuzzyF16) -> bool {
      let value = value.0;
      let mut buf = [0u8; f16::MAX_ENCODED_LEN + 1];
      let len = value.encoded_len();
      let len2 = value.encode(&mut buf).unwrap();
      assert_eq!(len, len2);
      let (read, value2) = f16::decode(&buf[..len]).unwrap();
      assert_eq!(len, read);
      assert_eq!(value, value2);

      encode_f16_varint(value).as_slice() == &buf[..len]
    }
  }

  #[cfg(feature = "std")]
  mod with_std {
    use super::*;

    quickcheck::quickcheck! {
      fn fuzzy_f16_sequence(value: std::vec::Vec<FuzzyF16>) -> bool {
        let value = value.into_iter().map(|v| v.0).collect::<std::vec::Vec<_>>();
        let encoded_len = encoded_f16_sequence_len(&value);
        let mut buf = std::vec![0; encoded_len];
        let Ok(written) = encode_f16_sequence_to(&value, &mut buf) else { return false; };
        if encoded_len != written {
          return false;
        }

        let (readed, decoded) = crate::decode_sequence::<f16, std::vec::Vec<_>>(&buf).unwrap();
        if encoded_len != readed {
          return false;
        }

        assert_eq!(decoded.len(), value.len());

        for (a, b) in decoded.iter().zip(value.iter()) {
          if a.to_bits() != b.to_bits() {
            return false;
          }
        }

        true
      }
    }
  }
}
