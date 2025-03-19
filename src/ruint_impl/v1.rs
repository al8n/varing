use crate::{DecodeError, EncodeError, Varint};

use ruint_1::Uint;

impl<const BITS: usize, const LBITS: usize> Varint for Uint<BITS, LBITS> {
  const MIN_ENCODED_LEN: usize = 1;
  const MAX_ENCODED_LEN: usize = BITS.div_ceil(7);

  fn encoded_len(&self) -> usize {
    // Each byte in LEB128 can store 7 bits
    // Special case for 0 since it always needs 1 byte
    if self.is_zero() {
      return 1;
    }

    // Calculate position of highest set bit
    let highest_bit = BITS - self.leading_zeros();
    // Convert to number of LEB128 bytes needed
    // Each byte holds 7 bits, but we need to round up
    highest_bit.div_ceil(7)
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    let len = self.encoded_len();
    let buf_len = buf.len();
    if buf_len < len {
      return Err(EncodeError::underflow(len, buf_len));
    }

    let mut value = *self;
    let mut bytes_written = 0;

    loop {
      let mut byte = (value & Uint::from(0x7f)).to::<u8>();
      value >>= 7;

      // If there are more bits to encode, set the continuation bit
      if !value.is_zero() {
        byte |= 0x80;
      }

      buf[bytes_written] = byte;
      bytes_written += 1;

      if value.is_zero() {
        break;
      }
    }

    Ok(bytes_written)
  }

  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    if buf.is_empty() {
      return Err(DecodeError::Underflow);
    }

    let mut result = Self::ZERO;
    let mut shift = 0;
    let mut bytes_read = 0;

    while bytes_read < buf.len() {
      let byte = buf[bytes_read];
      // Extract the 7 data bits
      let value = Self::from(byte & 0x7f);

      // Check for overflow
      if shift >= BITS {
        return Err(DecodeError::Overflow);
      }

      // Add the bits to the result
      // Need to handle potential overflow
      if let Some(shifted) = value.checked_shl(shift) {
        result |= shifted;
      } else {
        return Err(DecodeError::Overflow);
      }

      bytes_read += 1;

      // If continuation bit is not set, we're done
      if byte & 0x80 == 0 {
        return Ok((bytes_read, result));
      }

      shift += 7;
    }

    // If we get here, the input ended with a continuation bit set
    Err(DecodeError::Underflow)
  }
}

#[cfg(test)]
mod tests_ruint_1 {
  use super::*;

  type U256 = Uint<256, 4>;
  type U512 = Uint<512, 8>;
  type U1024 = Uint<1024, 16>;

  use quickcheck_macros::quickcheck;

  macro_rules! fuzzy {
    ($($ty:ident), +$(,)?) => {
      $(
        paste::paste! {
          #[quickcheck]
          fn [< fuzzy_ $ty:snake >](value: $ty) -> bool {
            let mut buf = [0; <$ty>::MAX_ENCODED_LEN];
            let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
            if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN) {
              return false;
            }

            if let Ok((bytes_read, decoded)) = <$ty>::decode(&buf) {
              value == decoded && encoded_len == bytes_read
            } else {
              false
            }
          }
        }
      )*
    };
  }

  #[test]
  fn test_max_encoded_len() {
    let value = U256::MAX;
    assert_eq!(value.encoded_len(), U256::MAX_ENCODED_LEN);
  }

  fuzzy!(U256, U512, U1024);

  #[cfg(feature = "std")]
  mod with_std {
    extern crate std;

    use super::*;

    use std::{vec, vec::Vec};

    // Helper type to generate fixed size arrays
    #[derive(Debug, Clone)]
    struct ByteArray<const N: usize>([u8; N]);

    impl<const N: usize> quickcheck::Arbitrary for ByteArray<N> {
      fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut arr = [0u8; N];
        for b in arr.iter_mut() {
          *b = u8::arbitrary(g);
        }
        ByteArray(arr)
      }
    }

    // Underflow tests for different sizes
    #[quickcheck]
    fn fuzzy_u256_buffer_underflow(bytes: ByteArray<32>, short_len: usize) -> bool {
      let uint = Uint::<256, 4>::from_be_bytes(bytes.0);
      let short_len = short_len % (Uint::<256, 4>::MAX_ENCODED_LEN - 1);
      if short_len >= uint.encoded_len() {
        return true;
      }
      let mut short_buffer = vec![0u8; short_len];
      matches!(
        uint.encode(&mut short_buffer),
        Err(EncodeError::Underflow { .. })
      )
    }

    #[quickcheck]
    fn fuzzy_u512_buffer_underflow(bytes: ByteArray<64>, short_len: usize) -> bool {
      let uint = Uint::<512, 8>::from_be_bytes(bytes.0);
      let short_len = short_len % (Uint::<512, 8>::MAX_ENCODED_LEN - 1);
      if short_len >= uint.encoded_len() {
        return true;
      }
      let mut short_buffer = vec![0u8; short_len];
      matches!(
        uint.encode(&mut short_buffer),
        Err(EncodeError::Underflow { .. })
      )
    }

    #[quickcheck]
    fn fuzzy_invalid_sequences(bytes: Vec<u8>) -> bool {
      if bytes.is_empty() {
        return matches!(U256::decode(&bytes), Err(DecodeError::Underflow));
      }

      // Only test sequences up to max varint length
      if bytes.len() > 10 {
        return true;
      }

      // If all bytes have continuation bit set, should get Underflow
      if bytes.iter().all(|b| b & 0x80 != 0) {
        return matches!(U256::decode(&bytes), Err(DecodeError::Underflow));
      }

      // For other cases, we should get either a valid decode or an error
      match U256::decode(&bytes) {
        Ok(_) => true,
        Err(_) => true,
      }
    }
  }
}
