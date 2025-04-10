#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

use core::{num::NonZeroU64, ops::RangeInclusive};

pub use char::*;
pub use duration::*;
use utils::zigzag_encode_i64;

/// Utilities for encoding and decoding LEB128 variable length integers.
pub mod utils;

mod char;
mod duration;

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
        return Err(DecodeError::Underflow);
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
          return Err(EncodeError::underflow([< encoded_ $ty _varint_len >](orig), $buf.len()));
        }

        $buf[i] = ($x as u8) | 0x80;
        $x >>= 7;
        i += 1;
      }

      // Check buffer capacity before writing final byte
      if i >= $buf.len() {
        return Err(EncodeError::underflow(i + 1, $buf.len()));
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

        #[doc = "Encodes an `i" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ i $ty _varint_to >](x: [< i $ty >], buf: &mut [u8]) -> Result<usize, EncodeError> {
          let mut x = utils::[< zigzag_encode_i $ty>](x);
          encode_varint!(@to_buf [<u $ty>]::buf[x])
        }
      }
    )*
  };
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

/// A trait for types that can be encoded as variable-length integers (varints).
///
/// Varints are a method of serializing integers using one or more bytes that allows small
/// numbers to be stored in fewer bytes. The encoding scheme is compatible with Protocol Buffers'
/// base-128 varint format.
pub trait Varint {
  /// The minimum number of bytes needed to encode any value of this type.
  ///
  /// - For `u16` and `i16`, this is `1`.
  /// - For `u32` and `i32`, this is `1`.
  /// - For `u64` and `i64`, this is `1`.
  /// - For `u128` and `i128`, this is `1`.
  const MIN_ENCODED_LEN: usize;

  /// The maximum number of bytes that might be needed to encode any value of this type.
  ///
  /// - For `u16` and `i16`, this is `3`.
  /// - For `u32` and `i32`, this is `5`.
  /// - For `u64` and `i64`, this is `10`.
  /// - For `u128` and `i128`, this is `19`.
  const MAX_ENCODED_LEN: usize;

  /// The range of possible encoded lengths for this type, from `MIN_ENCODED_LEN` to `MAX_ENCODED_LEN` inclusive.
  ///
  /// This range can be used to pre-allocate buffers or validate encoded data lengths.
  ///
  /// - For `u16` and `i16`, this range is `1..=3`, representing possible encoded lengths of 1, 2, or 3 bytes.
  /// - For `u32` and `i32`, this range is `1..=5`, representing possible encoded lengths of 1, 2, 3, 4, or 5 bytes.
  /// - For `u64` and `u64`, this range is `1..=10`, representing possible encoded lengths of 1 to 10 bytes.
  /// - For `u128` and `i128`, this range is `1..=19`, representing possible encoded lengths of 1 to 19 bytes.
  const ENCODED_LEN_RANGE: RangeInclusive<usize> = Self::MIN_ENCODED_LEN..=Self::MAX_ENCODED_LEN;

  /// Returns the encoded length of the value in LEB128 variable length format.
  /// The returned value will be in range [`Self::ENCODED_LEN_RANGE`](Varint::ENCODED_LEN_RANGE).
  fn encoded_len(&self) -> usize;

  /// Encodes the value as a varint and writes it to the buffer.
  ///
  /// Returns the number of bytes written to the buffer.
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;

  /// Decodes the value from the buffer.
  ///
  /// Returns the number of bytes read from the buffer and the decoded value if successful.
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized;
}

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

/// Calculates the number of bytes occupied by a varint encoded value in the buffer.
///
/// In varint encoding, each byte uses 7 bits for the value and the highest bit (MSB)
/// as a continuation flag. A set MSB (1) indicates more bytes follow, while an unset MSB (0)
/// marks the last byte of the varint.
///
/// ## Returns
/// * `Ok(usize)` - The number of bytes the varint occupies in the buffer
/// * `Err(DecodeError)` - If the buffer is empty or contains an incomplete varint
///
/// ## Examples
///
/// ```rust
/// use varing::consume_varint;
///
/// let buf = [0x96, 0x01]; // Varint encoding of 150
/// assert_eq!(consume_varint(&buf), Ok(2));
///
/// let buf = [0x7F]; // Varint encoding of 127
/// assert_eq!(consume_varint(&buf), Ok(1));
/// ```
pub const fn consume_varint(buf: &[u8]) -> Result<usize, DecodeError> {
  if buf.is_empty() {
    return Ok(0);
  }

  // Scan the buffer to find the end of the varint
  let mut idx = 0;
  let buf_len = buf.len();

  while idx < buf_len {
    let byte = buf[idx];
    // Check if this is the last byte of the varint (MSB is not set)
    if byte & 0x80 == 0 {
      // Found the last byte, return the total number of bytes
      return Ok(idx + 1);
    }

    // If we've reached the end of the buffer but haven't found the end of the varint
    if idx == buf_len - 1 {
      return Err(DecodeError::Underflow);
    }
    idx += 1;
  }

  // This point is reached only if all bytes have their MSB set and we've
  // exhausted the buffer, which means the varint is incomplete
  Err(DecodeError::Underflow)
}

/// Encode varint error
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeError {
  /// The buffer does not have enough capacity to encode the value.
  #[error("buffer does not have enough capacity to encode the value")]
  Underflow {
    /// The number of bytes needed to encode the value.
    required: usize,
    /// The number of bytes remaining in the buffer.
    remaining: usize,
  },
  /// A custom error message.
  #[error("{0}")]
  Custom(&'static str),
}

impl EncodeError {
  /// Creates a new `EncodeError::Underflow` with the required and remaining bytes.
  #[inline]
  pub const fn underflow(required: usize, remaining: usize) -> Self {
    Self::Underflow {
      required,
      remaining,
    }
  }

  /// Creates a new `EncodeError::Custom` with the given message.
  #[inline]
  pub const fn custom(msg: &'static str) -> Self {
    Self::Custom(msg)
  }
}

/// Decoding varint error.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum DecodeError {
  /// The buffer does not contain a valid LEB128 encoding.
  #[error("value would overflow the target type")]
  Overflow,
  /// The buffer does not contain enough data to decode.
  #[error("buffer does not contain enough data to decode a value")]
  Underflow,
  /// A custom error message.
  #[error("{0}")]
  Custom(&'static str),
}

impl DecodeError {
  /// Creates a new `DecodeError::Custom` with the given message.
  #[inline]
  pub const fn custom(msg: &'static str) -> Self {
    Self::Custom(msg)
  }
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
        return Err(DecodeError::custom("invalid boolean value"));
      }
      Ok((bytes_read, value != 0))
    })
  }
}

#[cfg(test)]
#[macro_use]
mod fuzz;

#[cfg(test)]
mod tests;

mod non_zero;

/// LEB128 encoding/decoding for `u1`, `u2` .. `u127`
#[cfg(feature = "arbitrary-int_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary-int")))]
pub mod arbitrary_int;

/// LEB128 encoding/decoding for [`num-rational`](https://crates.io/crates/num-rational) types.
#[cfg(feature = "num-rational_0_4")]
#[cfg_attr(docsrs, doc(cfg(feature = "num-rational_0_4")))]
pub mod num_rational;

/// LEB128 encoding/decoding for [`num-complex`](https://crates.io/crates/num-complex) types.
#[cfg(feature = "num-complex_0_4")]
#[cfg_attr(docsrs, doc(cfg(feature = "num-complex_0_4")))]
pub mod num_complex;

/// LEB128 encoding/decoding for [`bnum`](https://crates.io/crates/bnum) types.
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(feature = "bnum_0_13")))]
pub mod bnum;

/// LEB128 encoding/decoding for [`chrono`](https://crates.io/crates/chrono) types.
#[cfg(feature = "chrono_0_4")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono_0_4")))]
pub mod chrono;

/// LEB128 encoding/decoding for [`chrono-tz`](https://crates.io/crates/chrono-tz) types.
#[cfg(feature = "chrono-tz_0_10")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono-tz_0_10")))]
pub mod chrono_tz;

/// LEB128 encoding/decoding for [`time`](https://crates.io/crates/time) types.
#[cfg(feature = "time_0_3")]
#[cfg_attr(docsrs, doc(cfg(feature = "time_0_3")))]
pub mod time;

/// LEB128 encoding/decoding for [`primitive-types`](https://crates.io/crates/primitive-types) types.
#[cfg(feature = "primitive-types_0_13")]
#[cfg_attr(docsrs, doc(cfg(feature = "primitive-types_0_13")))]
pub mod primitive_types;

/// LEB128 encoding/decoding for [`ethereum-types`](https://crates.io/crates/ethereum-types) types.
#[cfg(feature = "ethereum-types_0_15")]
#[cfg_attr(docsrs, doc(cfg(feature = "ethereum-types_0_15")))]
pub mod ethereum_types;

/// Packable trait for types that can be packed into a single value.
pub mod packable;

#[cfg(feature = "ruint_1")]
mod ruint_impl;

#[cfg(any(feature = "chrono_0_4", feature = "time_0_3"))]
mod time_utils;
