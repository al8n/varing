use crate::*;

use ruint_1::Uint;

impl<const BITS: usize, const LBITS: usize> Varint for Uint<BITS, LBITS> {
  const MIN_ENCODED_LEN: NonZeroUsize = NON_ZERO_USIZE_ONE;
  const MAX_ENCODED_LEN: NonZeroUsize = {
    if BITS == 0 {
      NON_ZERO_USIZE_ONE
    } else {
      // Each byte can store 7 bits, round up
      // Safety: BITS > 0, so div_ceil(7) > 0
      NonZeroUsize::new(BITS.div_ceil(7)).unwrap()
    }
  };

  fn encoded_len(&self) -> NonZeroUsize {
    match BITS {
      0 => NON_ZERO_USIZE_ONE,
      1..=8 => encoded_u8_varint_len(self.to()),
      9..=16 => encoded_u16_varint_len(self.to()),
      17..=32 => encoded_u32_varint_len(self.to()),
      33..=64 => encoded_u64_varint_len(self.to()),
      65..=128 => encoded_u128_varint_len(self.to()),
      _ => {
        // Each byte in LEB128 can store 7 bits
        // Special case for 0 since it always needs 1 byte
        if self.is_zero() {
          return NON_ZERO_USIZE_ONE;
        }

        // Calculate position of highest set bit
        let highest_bit = BITS - self.leading_zeros();
        // Convert to number of LEB128 bytes needed
        // Each byte holds 7 bits, but we need to round up

        // Safety: if highest_bit is non-zero, div_ceil(7) is also non-zero
        unsafe { NonZeroUsize::new_unchecked(highest_bit.div_ceil(7)) }
      }
    }
  }

  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    if buf.is_empty() {
      return Err(EncodeError::insufficient_space(self.encoded_len(), 0));
    }

    match BITS {
      0 => {
        buf[0] = 0;
        Ok(NON_ZERO_USIZE_ONE)
      }
      1..=8 => encode_u8_varint_to(self.to(), buf).map_err(Into::into),
      9..=16 => encode_u16_varint_to(self.to(), buf).map_err(Into::into),
      17..=32 => encode_u32_varint_to(self.to(), buf).map_err(Into::into),
      33..=64 => encode_u64_varint_to(self.to(), buf).map_err(Into::into),
      65..=128 => encode_u128_varint_to(self.to(), buf).map_err(Into::into),
      _ => {
        let len = self.encoded_len();
        let buf_len = buf.len();
        if buf_len < len.get() {
          return Err(EncodeError::insufficient_space(len, buf_len));
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

        // Safety: bytes_written is guaranteed to be > 0 and <= buf_len
        Ok(unsafe { NonZeroUsize::new_unchecked(bytes_written) })
      }
    }
  }

  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    if buf.is_empty() {
      return Err(DecodeError::insufficient_data(buf.len()));
    }

    if BITS == 0 {
      if buf[0] != 0 {
        return Err(DecodeError::Overflow);
      }

      return Ok((NON_ZERO_USIZE_ONE, Self::ZERO));
    }

    let mut result = Self::ZERO;
    let mut shift = 0;
    let mut bytes_read = 0;

    while bytes_read < buf.len() {
      let byte = buf[bytes_read];
      // Extract the 7 data bits
      let payload = byte & 0x7f;

      // Check for overflow
      if shift >= BITS {
        return Err(DecodeError::Overflow);
      }

      // `checked_shl` only rejects shifts >= BITS; for a non-multiple-of-7
      // width the highest partial byte can carry data bits above BITS that
      // would otherwise be silently truncated. Reject those excess bits.
      let remaining_bits = BITS - shift;
      if remaining_bits < 7 && (payload >> remaining_bits) != 0 {
        return Err(DecodeError::Overflow);
      }

      let value = Self::from(payload);

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
        // Safety: bytes_read is guaranteed to be > 0 here
        return Ok((unsafe { NonZeroUsize::new_unchecked(bytes_read) }, result));
      }

      shift += 7;
    }

    // If we get here, the input ended with a continuation bit set
    Err(DecodeError::insufficient_data(buf.len()))
  }
}

#[cfg(any(feature = "num-rational_0_4", feature = "num-complex_0_4"))]
pub(crate) trait Packable<O> {
  fn pack(low: Self, high: Self) -> O;

  fn unpack(value: O) -> (Self, Self)
  where
    Self: Sized;
}

#[cfg(any(feature = "num-rational_0_4", feature = "num-complex_0_4"))]
impl<const BITS: usize, const LIMBS: usize, const OBITS: usize, const OLIMBS: usize>
  Packable<Uint<OBITS, OLIMBS>> for Uint<BITS, LIMBS>
{
  fn pack(low: Self, high: Self) -> Uint<OBITS, OLIMBS> {
    debug_assert_eq!(BITS * 2, OBITS, "BITS * 2 != OBITS");
    Uint::<OBITS, OLIMBS>::from(low) | (Uint::<OBITS, OLIMBS>::from(high) << Self::BITS)
  }

  fn unpack(value: Uint<OBITS, OLIMBS>) -> (Self, Self) {
    debug_assert_eq!(BITS * 2, OBITS, "BITS * 2 != OBITS");

    let low = value & Uint::<OBITS, OLIMBS>::from(Uint::<BITS, LIMBS>::MAX);
    let high = (value >> Self::BITS).to::<Uint<BITS, LIMBS>>();
    (low.to(), high.to())
  }
}

#[cfg(test)]
mod tests_ruint_1;
