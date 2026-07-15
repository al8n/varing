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
mod tests {
  use super::*;

  macro_rules! assert_bnum_width_boundary {
    ($digit_bits:literal) => {{
      let upstream_max_n = u32::MAX as u128 / $digit_bits as u128;
      let max_n = if upstream_max_n > usize::MAX as u128 {
        usize::MAX
      } else {
        upstream_max_n as usize
      };

      let bit_width = match checked_bnum_bit_width(max_n, $digit_bits) {
        Some(bit_width) => bit_width,
        None => panic!("largest supported bnum width was rejected"),
      };
      assert!(bit_width as u128 == max_n as u128 * $digit_bits as u128);

      if max_n < usize::MAX {
        assert!(checked_bnum_bit_width(max_n + 1, $digit_bits).is_none());
      }
    }};
  }

  const _: () = {
    assert_bnum_width_boundary!(8);
    assert_bnum_width_boundary!(16);
    assert_bnum_width_boundary!(32);
    assert_bnum_width_boundary!(64);

    if usize::BITS >= 32 {
      assert!(checked_bnum_bit_width(usize::MAX, 8).is_none());
      assert!(checked_bnum_bit_width(usize::MAX, 16).is_none());
      assert!(checked_bnum_bit_width(usize::MAX, 32).is_none());
      assert!(checked_bnum_bit_width(usize::MAX, 64).is_none());
    }
  };

  macro_rules! assert_terminal_boundary {
    ($storage:literal, $unsigned:ident, $signed:ident, $wire:expr) => {{
      paste::paste! {
        let wire = $wire;

        let (read, value) = [< decode_uint_d $storage >]::<1>(&wire).unwrap();
        assert_eq!(read.get(), wire.len());
        assert!(value == $unsigned::<1>::MAX);

        let mut encoded = [0u8; 10];
        let written = [< encode_uint_d $storage _to>]($unsigned::<1>::MAX, &mut encoded).unwrap();
        assert_eq!(&encoded[..written.get()], &wire);

        let (read, value) = [< decode_int_d $storage >]::<1>(&wire).unwrap();
        assert_eq!(read.get(), wire.len());
        assert!(value == $signed::<1>::MIN);

        let mut encoded = [0u8; 10];
        let written = [< encode_int_d $storage _to>]($signed::<1>::MIN, &mut encoded).unwrap();
        assert_eq!(&encoded[..written.get()], &wire);
      }
    }};
  }

  macro_rules! assert_overlong_zero {
    ($storage:literal, $unsigned:ident) => {{
      paste::paste! {
        let (read, value) = [< decode_ $unsigned:snake _d $storage >]::<1>(&[0x80, 0x00]).unwrap();
        assert_eq!(read.get(), 2);
        assert!(value.is_zero());
      }
    }};
  }

  macro_rules! assert_zero_width {
    ($storage:literal, $unsigned:ident) => {{
      paste::paste! {
        let unsigned = $unsigned::<0>::ZERO;
        assert_eq!([< encoded_uint_d $storage _len >](&unsigned).get(), 1);
        assert_eq!(<$unsigned<0> as Varint>::MAX_ENCODED_LEN.get(), 1);

        let mut encoded = [0xff];
        assert_eq!([< encode_uint_d $storage _to>](unsigned, &mut encoded).unwrap().get(), 1);
        assert_eq!(encoded, [0]);

        let (read, decoded) = [< decode_uint_d $storage >]::<0>(&[0]).unwrap();
        assert_eq!(read.get(), 1);
        assert!(decoded.is_zero());
        assert!(matches!([< decode_uint_d $storage >]::<0>(&[1]), Err(ConstDecodeError::Overflow)));
      }
    }};
  }

  #[test]
  fn terminal_payload_overflow_is_rejected() {
    let d8 = &[0x80, 0x02];
    assert!(matches!(
      decode_uint_d8::<1>(d8),
      Err(ConstDecodeError::Overflow)
    ));
    assert!(matches!(
      decode_int_d8::<1>(d8),
      Err(ConstDecodeError::Overflow)
    ));

    let d16 = &[0x80, 0x80, 0x04];
    assert!(matches!(
      decode_uint_d16::<1>(d16),
      Err(ConstDecodeError::Overflow)
    ));
    assert!(matches!(
      decode_int_d16::<1>(d16),
      Err(ConstDecodeError::Overflow)
    ));

    let d32 = &[0x80, 0x80, 0x80, 0x80, 0x10];
    assert!(matches!(
      decode_uint_d32::<1>(d32),
      Err(ConstDecodeError::Overflow)
    ));
    assert!(matches!(
      decode_int_d32::<1>(d32),
      Err(ConstDecodeError::Overflow)
    ));

    let d64 = &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02];
    assert!(matches!(
      decode_uint_d64::<1>(d64),
      Err(ConstDecodeError::Overflow)
    ));
    assert!(matches!(
      decode_int_d64::<1>(d64),
      Err(ConstDecodeError::Overflow)
    ));
  }

  #[test]
  fn terminal_payload_boundaries_round_trip() {
    assert_terminal_boundary!(8, BUintD8, BIntD8, [0xff, 0x01]);
    assert_terminal_boundary!(16, BUintD16, BIntD16, [0xff, 0xff, 0x03]);
    assert_terminal_boundary!(32, BUintD32, BIntD32, [0xff, 0xff, 0xff, 0xff, 0x0f]);
    assert_terminal_boundary!(
      64,
      BUint,
      BInt,
      [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
    );
  }

  #[test]
  fn overlong_representable_zero_is_accepted() {
    assert_overlong_zero!(8, uint);
    assert_overlong_zero!(16, uint);
    assert_overlong_zero!(32, uint);
    assert_overlong_zero!(64, uint);
    assert_overlong_zero!(8, int);
    assert_overlong_zero!(16, int);
    assert_overlong_zero!(32, int);
    assert_overlong_zero!(64, int);
  }

  #[test]
  fn zero_width_behavior_is_unchanged() {
    assert_zero_width!(8, BUintD8);
    assert_zero_width!(16, BUintD16);
    assert_zero_width!(32, BUintD32);
    assert_zero_width!(64, BUint);
  }

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
