use core::num::NonZeroU64;

use super::{
  utils::{self, zigzag_encode_i64},
  DecodeError, EncodeError, Varint,
};

macro_rules! impl_varint {
  ($($ty:literal), +$(,)?) => {
    $(
      paste::paste! {
        impl Varint for [< u $ty >] {
          const MIN_ENCODED_LEN: usize = [< encoded_ u $ty _varint_len >](0);
          const MAX_ENCODED_LEN: usize = [< encoded_ u $ty _varint_len >](<[< u $ty >]>::MAX);

          #[inline]
          fn encoded_len(&self) -> usize {
            [< encoded_ u $ty _varint_len >](*self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            [< encode_ u $ty _varint_to >](*self, buf)
          }

          #[inline]
          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError> {
            [< decode_ u $ty _varint >](buf)
          }
        }

        impl Varint for [< i $ty >] {
          const MIN_ENCODED_LEN: usize = [< encoded_ i $ty _varint_len >](0);
          const MAX_ENCODED_LEN: usize = [< encoded_ i $ty _varint_len >](<[< i $ty >]>::MAX);

          #[inline]
          fn encoded_len(&self) -> usize {
            [< encoded_ i $ty _varint_len >](*self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            [< encode_ i $ty _varint_to >](*self, buf)
          }

          #[inline]
          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError> {
            [< decode_ i $ty _varint >](buf)
          }
        }
      }
    )*
  };
}

macro_rules! decode_varint {
  (|$buf:ident| $ty:ident) => {{
    let mut result = 0;
    let mut shift = 0;
    let mut index = 0;

    loop {
      if index == $ty::MAX_ENCODED_LEN {
        return Err(DecodeError::Overflow);
      }

      if index >= $buf.len() {
        return Err(DecodeError::insufficient_data($buf.len()));
      }

      let next = $buf[index] as $ty;

      let v = $ty::BITS as usize / 7 * 7;
      let has_overflow = if shift < v {
        false
      } else if shift == v {
        next & ((u8::MAX << (::core::mem::size_of::<$ty>() % 7)) as $ty) != 0
      } else {
        true
      };

      if has_overflow {
        return Err(DecodeError::Overflow);
      }

      result += (next & 0x7F) << shift;
      if next & 0x80 == 0 {
        break;
      }
      shift += 7;
      index += 1;
    }
    Ok((index + 1, result))
  }};
}

macro_rules! encode_varint {
  ($buf:ident[$x:ident]) => {{
    let mut i = 0;

    while $x >= 0x80 {
      if i >= $buf.len() {
        panic!("insufficient buffer capacity");
      }

      $buf[i] = ($x as u8) | 0x80;
      $x >>= 7;
      i += 1;
    }

    // Check buffer capacity before writing final byte
    if i >= $buf.len() {
      panic!("insufficient buffer capacity");
    }

    $buf[i] = $x as u8;
    i + 1
  }};
  (@to_buf $ty:ident::$buf:ident[$x:ident]) => {{
    paste::paste! {
      let mut i = 0;
      let orig = $x;

      while $x >= 0x80 {
        if i >= $buf.len() {
          return Err(EncodeError::insufficient_space([< encoded_ $ty _varint_len >](orig), $buf.len()));
        }

        $buf[i] = ($x as u8) | 0x80;
        $x >>= 7;
        i += 1;
      }

      // Check buffer capacity before writing final byte
      if i >= $buf.len() {
        return Err(EncodeError::insufficient_space(i + 1, $buf.len()));
      }

      $buf[i] = $x as u8;
      Ok(i + 1)
    }
  }};
}

macro_rules! varint_len {
  ($($ty:ident),+$(,)?) => {
    $(
      paste::paste! {
        /// Returns the encoded length of the value in LEB128 variable length format.
        #[doc = "The returned value will be in range of [`" $ty "::ENCODED_LEN_RANGE`]."]
        #[inline]
        pub const fn [< encoded_ $ty _varint_len >](value: $ty) -> usize {
          encoded_u64_varint_len(value as u64)
        }
      }
    )*
  };
  (@zigzag $($ty:ident),+$(,)?) => {
    $(
      paste::paste! {
        /// Returns the encoded length of the value in LEB128 variable length format.
        #[doc = "The returned value will be in range of [`" $ty "::ENCODED_LEN_RANGE`]."]
        #[inline]
        pub const fn [< encoded_ $ty _varint_len >](value: $ty) -> usize {
          encoded_i64_varint_len(value as i64)
        }
      }
    )*
  };
}

macro_rules! encode {
  ($($ty:literal), +$(,)?) => {
    $(
      paste::paste! {
        #[doc = "Encodes an `u" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ u $ty _varint >](mut x: [< u $ty >]) -> $crate::utils::Buffer<{ [<u $ty>]::MAX_ENCODED_LEN + 1 }> {
          let mut buf = [0; [<u $ty>]::MAX_ENCODED_LEN + 1];
          let mut_buf = &mut buf;
          let len = encode_varint!(mut_buf[x]);
          buf[$crate::utils::Buffer::<{ [<u $ty>]::MAX_ENCODED_LEN + 1 }>::CAPACITY] = len as u8;
          $crate::utils::Buffer::new(buf)
        }

        #[doc = "Encodes an `i" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ i $ty _varint >](x: [< i $ty >]) -> $crate::utils::Buffer<{ [<u $ty>]::MAX_ENCODED_LEN + 1 }> {
          let x = utils::[< zigzag_encode_i $ty>](x);
          [< encode_ u $ty _varint >](x as [< u $ty >])
        }

        #[doc = "Encodes an `u" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ u $ty _varint_to >](mut x: [< u $ty >], buf: &mut [u8]) -> Result<usize, EncodeError> {
          encode_varint!(@to_buf [< u $ty >]::buf[x])
        }

        #[doc = "Returns the encoded length of a sequence of `u" $ty "` values"]
        #[inline]
        pub const fn [< encoded_ u $ty _sequence_len >](sequence: &[[< u $ty >]]) -> usize {
          encode!(@sequence_encoded_len_impl sequence, [< encoded_ u $ty _varint_len >])
        }

        #[doc = "Encodes a sequence of `u" $ty "` to the buffer."]
        #[inline]
        pub const fn [< encode_ u $ty _sequence_to >](sequence: &[[< u $ty >]], buf: &mut [u8]) -> Result<usize, EncodeError> {
          encode!(@sequence_encode_to_impl buf, sequence, [< encode_ u $ty _varint_to >], [< encoded_ u $ty _sequence_len >])
        }

        #[doc = "Encodes an `i" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ i $ty _varint_to >](x: [< i $ty >], buf: &mut [u8]) -> Result<usize, EncodeError> {
          let mut x = utils::[< zigzag_encode_i $ty>](x);
          encode_varint!(@to_buf [<u $ty>]::buf[x])
        }

        #[doc = "Returns the encoded length of a sequence of `i" $ty "` values"]
        #[inline]
        pub const fn [< encoded_i $ty _sequence_len >](sequence: &[[< i $ty >]]) -> usize {
          encode!(@sequence_encoded_len_impl sequence, [< encoded_ i $ty _varint_len >])
        }

        #[doc = "Encodes a sequence of `i" $ty "` to the buffer."]
        #[inline]
        pub const fn [< encode_i $ty _sequence_to >](sequence: &[[< i $ty >]], buf: &mut [u8]) -> Result<usize, EncodeError> {
          encode!(@sequence_encode_to_impl buf, sequence, [< encode_ i $ty _varint_to >], [< encoded_ i $ty _sequence_len >])
        }
      }
    )*
  };
  (@sequence_encode_to_impl $buf:ident, $sequence:ident, $encode_to:ident, $encoded_sequence_len:ident) => {{
    let mut total_bytes = 0;
    let mut idx = 0;
    let len = $sequence.len();
    let buf_len = $buf.len();

    while idx < len && total_bytes < buf_len {
      let (_, buf) = $buf.split_at_mut(total_bytes);
      let bytes_written = match $encode_to($sequence[idx], buf) {
        Ok(bytes_written) => bytes_written,
        Err(e) => return Err(e.update($encoded_sequence_len($sequence), buf_len)),
      };
      total_bytes += bytes_written;
      idx += 1;
    }

    Ok(total_bytes)
  }};
  (@sequence_encoded_len_impl $sequence:ident, $encoded_len:ident) => {{
    let mut total_bytes = 0;
    let mut idx = 0;
    let len = $sequence.len();

    while idx < len {
      total_bytes += $encoded_len($sequence[idx]);
      idx += 1;
    }

    total_bytes
  }};
}

macro_rules! decode {
  ($($ty:literal), + $(,)?) => {
    $(
      paste::paste! {
        #[doc = "Decodes an `i" $ty "` in LEB128 encoded format from the buffer."]
        ///
        /// Returns the bytes readed and the decoded value if successful.
        pub const fn [< decode_ u $ty _varint >](buf: &[u8]) -> Result<(usize, [< u $ty >]), DecodeError> {
          decode_varint!(|buf| [< u $ty >])
        }

        #[doc = "Decodes an `u" $ty "` in LEB128 encoded format from the buffer."]
        ///
        /// Returns the bytes readed and the decoded value if successful.
        pub const fn [< decode_ i $ty _varint >](buf: &[u8]) -> Result<(usize, [< i $ty >]), DecodeError> {
          match [< decode_ u $ty _varint >](buf) {
            Ok((bytes_read, value)) => {
              let value = utils::[<zigzag_decode_i $ty>](value);
              Ok((bytes_read, value))
            },
            Err(e) => Err(e),
          }
        }
      }
    )*
  };
}

impl_varint!(8, 16, 32, 64, 128,);
varint_len!(u8, u16, u32,);
varint_len!(@zigzag i8, i16, i32,);
encode!(128, 64, 32, 16, 8);
decode!(128, 64, 32, 16, 8);

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`u128::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_u128_varint_len(value: u128) -> usize {
  // Each byte in LEB128 encoding can hold 7 bits of data
  // We want to find how many groups of 7 bits are needed
  // Special case for 0 and small numbers
  if value < 128 {
    return 1;
  }

  // Calculate position of highest set bit
  let highest_bit = 128 - value.leading_zeros();
  // Convert to number of LEB128 bytes needed
  // Each byte holds 7 bits, but we need to round up
  highest_bit.div_ceil(7) as usize
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`i128::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_i128_varint_len(x: i128) -> usize {
  let x = utils::zigzag_encode_i128(x);
  encoded_u128_varint_len(x)
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`i64::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_i64_varint_len(x: i64) -> usize {
  let x = zigzag_encode_i64(x);
  encoded_u64_varint_len(x)
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`u64::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_u64_varint_len(value: u64) -> usize {
  // Based on [VarintSize64][1].
  // [1]: https://github.com/protocolbuffers/protobuf/blob/v28.3/src/google/protobuf/io/coded_stream.h#L1744-L1756
  // Safety: (value | 1) is never zero
  let log2value = unsafe { NonZeroU64::new_unchecked(value | 1) }.ilog2();
  ((log2value * 9 + (64 + 9)) / 64) as usize
}

impl Varint for bool {
  const MIN_ENCODED_LEN: usize = 1;

  const MAX_ENCODED_LEN: usize = 1;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_u8_varint_len(*self as u8)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_u8_varint_to(*self as u8, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_u8_varint(buf).and_then(|(bytes_read, value)| {
      if value > 1 {
        return Err(DecodeError::other("invalid boolean value"));
      }
      Ok((bytes_read, value != 0))
    })
  }
}

impl Varint for f32 {
  const MIN_ENCODED_LEN: usize = u32::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: usize = u32::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_f32_varint_len(*self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_f32_varint_to(*self, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_f32_varint(buf)
  }
}

impl Varint for f64 {
  const MIN_ENCODED_LEN: usize = u64::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: usize = u64::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_f64_varint_len(*self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_f64_varint_to(*self, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_f64_varint(buf)
  }
}

/// Returns the encoded length of the value in LEB128 variable length format. The returned value will be in range of [`f32::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_f32_varint_len(value: f32) -> usize {
  crate::encoded_u32_varint_len(value.to_bits())
}

/// Encodes an `f32` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f32_varint(value: f32) -> crate::utils::Buffer<{ f32::MAX_ENCODED_LEN + 1 }> {
  crate::encode_u32_varint(value.to_bits())
}

/// Encodes an `f32` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f32_varint_to(value: f32, buf: &mut [u8]) -> Result<usize, crate::EncodeError> {
  crate::encode_u32_varint_to(value.to_bits(), buf)
}

/// Decodes an `f32` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_f32_varint(buf: &[u8]) -> Result<(usize, f32), crate::DecodeError> {
  match crate::decode_u32_varint(buf) {
    Ok((len, bits)) => Ok((len, f32::from_bits(bits))),
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format. The returned value will be in range of [`f64::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_f64_varint_len(value: f64) -> usize {
  crate::encoded_u64_varint_len(value.to_bits())
}

/// Encodes an `f64` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f64_varint(value: f64) -> crate::utils::Buffer<{ f64::MAX_ENCODED_LEN + 1 }> {
  crate::encode_u64_varint(value.to_bits())
}

/// Encodes an `f64` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_f64_varint_to(value: f64, buf: &mut [u8]) -> Result<usize, crate::EncodeError> {
  crate::encode_u64_varint_to(value.to_bits(), buf)
}

/// Decodes an `f64` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_f64_varint(buf: &[u8]) -> Result<(usize, f64), crate::DecodeError> {
  match crate::decode_u64_varint(buf) {
    Ok((len, bits)) => Ok((len, f64::from_bits(bits))),
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of a sequence of `f32` values
#[inline]
pub const fn encoded_f32_sequence_len(sequence: &[f32]) -> usize {
  encode!(@sequence_encoded_len_impl sequence, encoded_f32_varint_len)
}

/// Encodes a sequence of `f32` to the buffer.
#[inline]
pub const fn encode_f32_sequence_to(
  sequence: &[f32],
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  encode!(@sequence_encode_to_impl buf, sequence, encode_f32_varint_to, encoded_f32_sequence_len)
}

/// Returns the encoded length of a sequence of `f64` values
#[inline]
pub const fn encoded_f64_sequence_len(sequence: &[f64]) -> usize {
  encode!(@sequence_encoded_len_impl sequence, encoded_f64_varint_len)
}

/// Encodes a sequence of `f64` to the buffer.
#[inline]
pub const fn encode_f64_sequence_to(
  sequence: &[f64],
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  encode!(@sequence_encode_to_impl buf, sequence, encode_f64_varint_to, encoded_f64_sequence_len)
}

/// LEB128 encoding/decoding for [`half`](https://crates.io/crates/half) types.
#[cfg(feature = "half_2")]
mod half;
#[cfg(feature = "half_2")]
pub use half::*;
