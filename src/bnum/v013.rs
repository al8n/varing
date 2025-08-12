use bnum_0_13::{BInt, BIntD16, BIntD32, BIntD8, BUint, BUintD16, BUintD32, BUintD8};

use crate::{ConstDecodeError, ConstEncodeError, Varint};

use core::num::NonZeroUsize;

macro_rules! unsigned {
  ($($base:ident($storage:literal)), +$(,)?) => {
    paste::paste! {
      $(
        /// Returns the encoded length of the value in LEB128 variable length format.
        #[doc = "The returned value will be in range of [`" $base "::<N>::ENCODED_LEN_RANGE`](Varint::ENCODED_LEN_RANGE)."]
        #[inline]
        pub const fn [< encoded_uint_d $storage _len >]<const N: usize>(val: &$base<N>) -> NonZeroUsize {
          if N == 0 {
            return crate::NON_ZERO_USIZE_ONE;
          }

          // Each byte in LEB128 can store 7 bits
          // Special case for 0 since it always needs 1 byte
          if val.is_zero() {
            return crate::NON_ZERO_USIZE_ONE;
          }

          // Calculate position of highest set bit
          let highest_bit = ((N * $storage) as u32) - val.leading_zeros();
          // Convert to number of LEB128 bytes needed
          // Each byte holds 7 bits, but we need to round up
          // Safety: highest_bit is non-zero, div_ceil(7) is also non-zero
          unsafe { NonZeroUsize::new_unchecked(highest_bit.div_ceil(7) as usize) }
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
          if buf.is_empty() {
            return Err(ConstDecodeError::insufficient_data(buf.len()));
          }

          if N == 0 {
            return if buf[0] == 0 {
              Ok((crate::NON_ZERO_USIZE_ONE, $base::<N>::ZERO))
            } else {
              Err(ConstDecodeError::Overflow)
            };
          }

          let mut result = $base::<N>::ZERO;
          let mut shift = 0;
          let mut bytes_read = 0;

          while bytes_read < buf.len() {
            let byte = buf[bytes_read];
            // Extract the 7 data bits
            let value = $base::<N>::from_digit((byte & 0x7f).to_le() as [< u$storage >]);

            // Check for overflow
            if shift >= N * $storage {
              return Err(ConstDecodeError::Overflow);
            }

            // Add the bits to the result
            // Need to handle potential overflow
            if let Some(shifted) = value.checked_shl(shift as u32) {
              result = result.bitor(shifted);
            } else {
              return Err(ConstDecodeError::Overflow);
            }

            bytes_read += 1;

            // If continuation bit is not set, we're done
            if byte & 0x80 == 0 {
              // Safety: bytes_read is guaranteed to be > 0 and <= buf_len
              return Ok((unsafe { NonZeroUsize::new_unchecked(bytes_read) }, result));
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
            NonZeroUsize::new((N * $storage).div_ceil(7)).unwrap()
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
          if N == 0 {
            return $u::<N>::ZERO;
          }

          let bits = match (N * $storage).checked_sub(1) {
            Some(val) => val,
            None => 0,
          };

          value.shl(1).bitxor(value.shr(bits as u32)).to_bits()
        }

        #[doc = "Zigzag decode `" $i "<N>` value"]
        #[inline]
        pub const fn [< zigzag_decode_int_d $storage >]<const N: usize>(value: &$u<N>) -> $i<N> {
          if N == 0 {
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
          if N == 0 {
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
mod tests {
  use super::*;

  macro_rules! fuzzy {
    ($base:ident($($ty:ident), +$(,)?)) => {
      $(
        paste::paste! {
          #[quickcheck_macros::quickcheck]
          fn [< fuzzy_$base:snake _ $ty:snake >](value: $ty) -> bool {
            let mut buf = [0; <$ty>::MAX_ENCODED_LEN.get()];
            let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
            if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN) {
              return false;
            }

            let Some(consumed) = crate::consume_varint_checked(&buf) else {
              return false;
            };
            if consumed != encoded_len {
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

  macro_rules! define_aliases {
    ($sign:ident::$base:ident ($($ty:literal), +$(,)?)) => {
      paste::paste! {
        $(
          type [< $sign:camel $ty >] = $base<$ty>;
        )*
      }
    };
  }

  macro_rules! fuzzy_mod {
    ($(mod $mod_name:ident ($sign:ident::$base:ident($start:literal..=$end:literal))),+$(,)?) => {
      paste::paste! {
        $(
          mod $mod_name {
            use super::*;

            seq_macro::seq!(
              N in $start..=$end {
                define_aliases!($sign::$base(#(N,)*));

                fuzzy!($base(#([< $sign:camel >]~N,)*));
              }
            );
          }
        )*
      }
    };
  }

  fuzzy_mod! {
    mod buint_d8 (u::BUintD8(0..=64)),
    mod buint_d16 (u::BUintD16(0..=64)),
    mod buint_d32 (u::BUintD32(0..=64)),
    mod buint(u::BUint(0..=64)),
    mod bint_d8 (i::BIntD8(1..=64)),
    mod bint_d16 (i::BIntD16(1..=64)),
    mod bint_d32 (i::BIntD32(1..=64)),
    mod bint(i::BInt(1..=64)),
  }
}
