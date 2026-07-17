use bnum_0_13::{BInt, BIntD8, BIntD16, BIntD32, BUint, BUintD8, BUintD16, BUintD32};

use crate::{ConstDecodeError, ConstEncodeError, Varint};

use core::num::NonZeroUsize;

#[inline]
const fn checked_bnum_bit_width(n: usize, digit_bits: u32) -> Option<u32> {
  if digit_bits == 0 || n as u128 > u32::MAX as u128 / digit_bits as u128 {
    None
  } else {
    Some((n as u32) * digit_bits)
  }
}

#[inline]
const fn bnum_bit_width(n: usize, digit_bits: u32) -> u32 {
  match checked_bnum_bit_width(n, digit_bits) {
    Some(bit_width) => bit_width,
    None => panic!("varing: bnum 0.13 varint width exceeds its u32 bit-width limit"),
  }
}

macro_rules! unsigned {
  ($($base:ident($storage:literal)), +$(,)?) => {
    paste::paste! {
      $(
        /// Returns the encoded length of the value in LEB128 variable length format.
        #[doc = "The returned value will be in range of [`" $base "::<N>::ENCODED_LEN_RANGE`](Varint::ENCODED_LEN_RANGE)."]
        #[inline]
        pub const fn [< encoded_uint_d $storage _len >]<const N: usize>(val: &$base<N>) -> NonZeroUsize {
          let bit_width = bnum_bit_width(N, $storage);
          if bit_width == 0 {
            return crate::NON_ZERO_USIZE_ONE;
          }

          // Each byte in LEB128 can store 7 bits
          // Special case for 0 since it always needs 1 byte
          if val.is_zero() {
            return crate::NON_ZERO_USIZE_ONE;
          }

          // Calculate position of highest set bit
          let highest_bit = bit_width - val.leading_zeros();
          // Convert to number of LEB128 bytes needed
          // Each byte holds 7 bits, but we need to round up
          NonZeroUsize::new(highest_bit.div_ceil(7) as usize).unwrap()
        }

        #[doc = "Encodes an `" $base "<N>` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_uint_d $storage _to >]<const N: usize>(
          mut value: $base<N>,
          buf: &mut [u8],
        ) -> Result<NonZeroUsize, ConstEncodeError> {
          let len = [< encoded_uint_d $storage _len >](&value);
          let buf_len = buf.len();
          if buf_len < len.get() {
            return Err(ConstEncodeError::insufficient_space(len, buf_len));
          }

          if N == 0 {
            buf[0] = 0;
            return Ok(crate::NON_ZERO_USIZE_ONE);
          }

          let mut bytes_written = 0;
          loop {
            let mut byte = value.bitand($base::<N>::from_digit([< 0x7fu $storage >].to_le())).digits()[0] as u8;
            value = value.shr(7);

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

          // Safety: bytes_written is guaranteed to be non-zero
          Ok(unsafe { NonZeroUsize::new_unchecked(bytes_written) })
        }

        #[doc = "Decodes an `" $base "<N>` in LEB128 encoded format from the buffer."]
        ///
        /// Returns the bytes readed and the decoded value if successful.
        pub const fn [< decode_uint_d $storage >]<const N: usize>(
          buf: &[u8],
        ) -> Result<(NonZeroUsize, $base<N>), ConstDecodeError> {
          let bit_width = bnum_bit_width(N, $storage);

          if buf.is_empty() {
            return Err(ConstDecodeError::insufficient_data(buf.len()));
          }

          if bit_width == 0 {
            return if buf[0] == 0 {
              Ok((crate::NON_ZERO_USIZE_ONE, $base::<N>::ZERO))
            } else {
              Err(ConstDecodeError::Overflow)
            };
          }

          let mut result = $base::<N>::ZERO;
          let mut shift = 0u32;
          let mut bytes_read = 0;

          while bytes_read < buf.len() {
            let byte = buf[bytes_read];
            // Extract the 7 data bits
            let payload = byte & 0x7f;

            // Check for overflow
            if shift >= bit_width {
              return Err(ConstDecodeError::Overflow);
            }

            let remaining_bits = bit_width - shift;
            if remaining_bits < 7 && payload >> remaining_bits != 0 {
              return Err(ConstDecodeError::Overflow);
            }

            let value = $base::<N>::from_digit(payload.to_le() as [< u$storage >]);

            // Add the bits to the result
            if let Some(shifted) = value.checked_shl(shift) {
              result = result.bitor(shifted);
            } else {
              return Err(ConstDecodeError::Overflow);
            }

            bytes_read += 1;

            // If continuation bit is not set, we're done
            if byte & 0x80 == 0 {
              return Ok((NonZeroUsize::new(bytes_read).unwrap(), result));
            }

            shift += 7;
          }

          // If we get here, the input ended with a continuation bit set
          Err(ConstDecodeError::insufficient_data(buf.len()))
        }

        impl<const N: usize> Varint for $base<N> {
          const MIN_ENCODED_LEN: NonZeroUsize = crate::NON_ZERO_USIZE_ONE;

          const MAX_ENCODED_LEN: NonZeroUsize = { if N == 0 {
            crate::NON_ZERO_USIZE_ONE
          } else {
            NonZeroUsize::new((bnum_bit_width(N, $storage) as usize).div_ceil(7)).unwrap()
          } };

          fn encoded_len(&self) -> NonZeroUsize {
            [< encoded_uint_d $storage _len >](self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, crate::EncodeError> {
            [< encode_uint_d $storage _to >](*self, buf).map_err(Into::into)
          }

          fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), crate::DecodeError>
          where
            Self: Sized,
          {
            [< decode_uint_d $storage >](buf).map_err(Into::into)
          }
        }
      )*
    }
  };
}

macro_rules! signed {
  ($($storage:literal($i:ident <=> $u:ident)), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Zigzag encode `" $i "<N>` value"]
        #[inline]
        pub const fn [< zigzag_encode_int_d $storage >]<const N: usize>(value: &$i<N>) -> $u<N> {
          let bit_width = bnum_bit_width(N, $storage);
          if bit_width == 0 {
            return $u::<N>::ZERO;
          }

          value.shl(1).bitxor(value.shr(bit_width - 1)).to_bits()
        }

        #[doc = "Zigzag decode `" $i "<N>` value"]
        #[inline]
        pub const fn [< zigzag_decode_int_d $storage >]<const N: usize>(value: &$u<N>) -> $i<N> {
          if bnum_bit_width(N, $storage) == 0 {
            return $i::<N>::ZERO;
          }

          let a = $i::<N>::from_bits(value.shr(1));
          let b = $i::<N>::from_bits(value.bitand($u::<N>::from_digit(1))).neg();
          a.bitxor(b)
        }

        /// Returns the encoded length of the value in LEB128 variable length format.
        #[doc = "The returned value will be in range of [`" $i "::<N>::ENCODED_LEN_RANGE`](Varint::ENCODED_LEN_RANGE)."]
        #[inline]
        pub const fn [< encoded_int_d $storage _len >]<const N: usize>(val: &$i<N>) -> NonZeroUsize {
          if N == 0 {
            return crate::NON_ZERO_USIZE_ONE;
          }

          [< encoded_uint_d $storage _len >](&[< zigzag_encode_int_d $storage>](&val))
        }

        #[doc = "Encodes an `" $i "<N>` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_int_d $storage _to >]<const N: usize>(
          value: $i<N>,
          buf: &mut [u8],
        ) -> Result<NonZeroUsize, ConstEncodeError> {
          if N == 0 {
            if buf.is_empty() {
              return Err(ConstEncodeError::insufficient_space(crate::NON_ZERO_USIZE_ONE, 0));
            }
            buf[0] = 0;
            return Ok(crate::NON_ZERO_USIZE_ONE);
          }

          [< encode_uint_d $storage _to>]([< zigzag_encode_int_d $storage>](&value), buf)
        }

        #[doc = "Decodes an `" $i "<N>` in LEB128 encoded format from the buffer."]
        ///
        /// Returns the bytes readed and the decoded value if successful.
        pub const fn [< decode_int_d $storage >]<const N: usize>(
          buf: &[u8],
        ) -> Result<(NonZeroUsize, $i<N>), ConstDecodeError> {
          let bit_width = bnum_bit_width(N, $storage);

          if bit_width == 0 {
            if buf.is_empty() {
              return Err(ConstDecodeError::insufficient_data(buf.len()));
            }

            if buf[0] != 0 {
              return Err(ConstDecodeError::Overflow);
            }

            return Ok((crate::NON_ZERO_USIZE_ONE, $i::<N>::ZERO));
          }

          if buf.is_empty() {
            return Err(ConstDecodeError::insufficient_data(buf.len()));
          }

          match [< decode_uint_d $storage >]::<N>(buf) {
            Ok((read, val)) => {
              let val = [<zigzag_decode_int_d $storage>](&val);
              Ok((read, val))
            },
            Err(e) => Err(e),
          }
        }

        impl<const N: usize> Varint for $i<N> {
          const MIN_ENCODED_LEN: NonZeroUsize = $u::<N>::MIN_ENCODED_LEN;

          const MAX_ENCODED_LEN: NonZeroUsize = $u::<N>::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> NonZeroUsize {
            [< encoded_int_d $storage _len >](self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, crate::EncodeError> {
            [< encode_int_d $storage _to >](*self, buf).map_err(Into::into)
          }

          fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), crate::DecodeError>
          where
            Self: Sized,
          {
            [< decode_int_d $storage >](buf).map_err(Into::into)
          }
        }
      )*
    }
  };
}

unsigned!(BUintD8(8), BUintD16(16), BUintD32(32), BUint(64));
signed!(8(BIntD8 <=> BUintD8), 16(BIntD16 <=> BUintD16), 32(BIntD32 <=> BUintD32), 64(BInt <=> BUint));

#[cfg(test)]
mod tests;
