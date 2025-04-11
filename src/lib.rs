#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

use core::ops::RangeInclusive;

pub use char::*;
pub use duration::*;
pub use primitives::*;

/// Utilities for encoding and decoding LEB128 variable length integers.
pub mod utils;

mod char;
mod duration;
mod primitives;

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

/// Encodes a sequence of values as varints and writes them to the buffer.
///
/// Returns the total number of bytes written to the buffer.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "std")]
/// # {
/// #
/// use varing::{Varint, encoded_sequence_len, encode_sequence, decode_sequence};
///
/// let values = (0..1024u64).collect::<Vec<_>>();
///
/// let encoded_len = encoded_sequence_len(values.iter());
/// let mut buf = vec![0; encoded_len];
///
/// let bytes_written = encode_sequence(values.iter(), &mut buf).unwrap();
/// assert_eq!(bytes_written, encoded_len);
///
/// let (readed, decoded) = decode_sequence::<u64, Vec<u64>>(&buf).unwrap();
///
/// assert_eq!(decoded, values);
/// assert_eq!(readed, buf.len());
/// # }
/// ```
pub fn encode_sequence<'i, V>(
  sequence: impl Iterator<Item = &'i V>,
  buf: &mut [u8],
) -> Result<usize, EncodeError>
where
  V: ?Sized + Varint + 'i,
{
  let mut total_bytes = 0;
  for value in sequence {
    let bytes_written = value.encode(&mut buf[total_bytes..])?;
    total_bytes += bytes_written;
  }
  Ok(total_bytes)
}

/// Returns the total number of bytes needed to encode a sequence of values.
pub fn encoded_sequence_len<'i, V>(sequence: impl Iterator<Item = &'i V>) -> usize
where
  V: ?Sized + Varint + 'i,
{
  sequence.map(|item| item.encoded_len()).sum::<usize>()
}

/// Returns a sequence decoder for the given buffer.
///
/// The returned decoder is an iterator that yields `Result<(usize, Self), DecodeError>`.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "std")]
/// # {
/// #
/// use varing::{Varint, encoded_sequence_len, encode_sequence, sequence_decoder};
///
/// let values = (0..1024u64).collect::<Vec<_>>();
///
/// let encoded_len = encoded_sequence_len(values.iter());
/// let mut buf = vec![0; encoded_len];
///
/// let bytes_written = encode_sequence(values.iter(), &mut buf).unwrap();
/// assert_eq!(bytes_written, encoded_len);
///
/// let mut readed = 0;
/// let mut decoded = Vec::new();
/// let mut decoder = sequence_decoder::<u64>(&buf[readed..]);
/// // `SequenceDecoder` is copy
/// let mut decoder1 = decoder;
/// while let Some(Ok((bytes_read, value))) = decoder.next() {
///   readed += bytes_read;
///   assert_eq!(readed, decoder.position());
///   decoded.push(value);
/// }
///
/// assert_eq!(decoder1.position(), 0);
/// assert_eq!(decoded, values);
/// assert_eq!(readed, buf.len());
/// # }
/// ```
#[inline]
pub const fn sequence_decoder<V>(buf: &[u8]) -> SequenceDecoder<'_, V>
where
  V: ?Sized,
{
  SequenceDecoder::new(buf)
}

/// Decodes a sequence of values from the buffer.
///
/// Returns the number of bytes read from the buffer and a collection of the decoded value if successful.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "std")]
/// # {
/// #
/// use varing::{Varint, encoded_sequence_len, encode_sequence, decode_sequence};
///
/// let values = (0..1024u64).collect::<Vec<_>>();
///
/// let encoded_len = encoded_sequence_len(values.iter());
/// let mut buf = vec![0; encoded_len];
///
/// let bytes_written = encode_sequence(values.iter(), &mut buf).unwrap();
/// assert_eq!(bytes_written, encoded_len);
///
/// let (readed, decoded) = decode_sequence::<u64, Vec<_>>(&buf).unwrap();
///
/// assert_eq!(decoded, values);
/// assert_eq!(readed, buf.len());
/// # }
/// ```
pub fn decode_sequence<V, O>(buf: &[u8]) -> Result<(usize, O), DecodeError>
where
  V: Varint,
  O: core::iter::FromIterator<V>,
{
  let mut readed = 0;
  core::iter::from_fn(|| {
    if readed < buf.len() {
      match V::decode(&buf[readed..]) {
        Ok((bytes_read, value)) => {
          readed += bytes_read;
          Some(Ok(value))
        }
        Err(e) => Some(Err(e)),
      }
    } else {
      None
    }
  })
  .collect::<Result<O, _>>()
  .map(|output| (readed, output))
}

/// Encodes a map of entries as varints and writes them to the buffer.
///
/// Returns the total number of bytes written to the buffer.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "std")]
/// # {
/// #
/// use varing::{Varint, decode_map, encoded_map_len, encode_map};
/// use std::collections::HashMap;
///
/// let values = (0..1024u64).map(|v| (v, v)).collect::<HashMap<_, _>>();
///
/// let encoded_len = encoded_map_len(values.iter());
/// let mut buf = vec![0; encoded_len];
///
/// let bytes_written = encode_map(values.iter(), &mut buf).unwrap();
/// assert_eq!(bytes_written, encoded_len);
///
/// let (readed, decoded) = decode_map::<_, _, HashMap<u64, u64>>(&buf).unwrap();
///
/// assert_eq!(decoded, values);
/// assert_eq!(readed, buf.len());
/// # }
/// ```
pub fn encode_map<'a, K, V>(
  map: impl Iterator<Item = (&'a K, &'a V)>,
  buf: &mut [u8],
) -> Result<usize, EncodeError>
where
  K: Varint + 'a,
  V: Varint + 'a,
{
  let mut total_bytes = 0;
  for (key, value) in map {
    let bytes_written = key.encode(&mut buf[total_bytes..])?;
    total_bytes += bytes_written;
    let bytes_written = value.encode(&mut buf[total_bytes..])?;
    total_bytes += bytes_written;
  }
  Ok(total_bytes)
}

/// Returns the total length of a map of entries.
///
/// Returns the total number of bytes needed to encode the map.
pub fn encoded_map_len<'a, K, V>(map: impl Iterator<Item = (&'a K, &'a V)>) -> usize
where
  K: Varint + 'a,
  V: Varint + 'a,
{
  map
    .map(|(key, value)| key.encoded_len() + value.encoded_len())
    .sum::<usize>()
}

/// Returns a sequence decoder for the given buffer.
///
/// The returned decoder is an iterator that yields `Result<(usize, Self), DecodeError>`.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "std")]
/// # {
/// #
/// use varing::{Varint, decode_map, encoded_map_len, encode_map, map_decoder};
/// use std::collections::HashMap;
///
/// let values = (0..1024u64).map(|v| (v, v)).collect::<HashMap<_, _>>();
///
/// let encoded_len = encoded_map_len(values.iter());
/// let mut buf = vec![0; encoded_len];
///
/// let bytes_written = encode_map(values.iter(), &mut buf).unwrap();
/// assert_eq!(bytes_written, encoded_len);
///
/// let mut readed = 0;
/// let mut decoded = HashMap::new();
/// let mut decoder = map_decoder::<u64, u64>(&buf[readed..]);
/// // `MapDecoder` is copy
/// let mut decoder1 = decoder;
/// while let Some(Ok((bytes_read, (key, value)))) = decoder.next() {
///   readed += bytes_read;
///   assert_eq!(readed, decoder.position());
///   decoded.insert(key, value);
/// }
///
/// assert_eq!(decoder1.position(), 0);
/// assert_eq!(decoded, values);
/// assert_eq!(readed, buf.len());
/// # }
/// ```
pub const fn map_decoder<K, V>(buf: &[u8]) -> MapDecoder<'_, K, V>
where
  K: ?Sized,
  V: ?Sized,
{
  MapDecoder::new(buf)
}

/// Decodes a collection of entries from the buffer.
///
/// Returns the number of bytes read from the buffer and a collection of the decoded value if successful.
///
/// ## Example
///
/// ```rust
/// # #[cfg(feature = "std")]
/// # {
/// #
/// use varing::{Varint, decode_map, encoded_map_len, encode_map};
/// use std::collections::HashMap;
///
/// let values = (0..1024u64).map(|v| (v, v)).collect::<HashMap<_, _>>();
///
/// let encoded_len = encoded_map_len(values.iter());
/// let mut buf = vec![0; encoded_len];
///
/// let bytes_written = encode_map(values.iter(), &mut buf).unwrap();
/// assert_eq!(bytes_written, encoded_len);
///
/// let (readed, decoded) = decode_map::<_, _, HashMap<u64, u64>>(&buf).unwrap();
///
/// assert_eq!(decoded, values);
/// assert_eq!(readed, buf.len());
/// # }
/// ```
pub fn decode_map<K, V, O>(buf: &[u8]) -> Result<(usize, O), DecodeError>
where
  K: Varint + Sized,
  V: Varint + Sized,
  O: core::iter::FromIterator<(K, V)>,
{
  let mut readed = 0;
  core::iter::from_fn(|| {
    if readed < buf.len() {
      Some(K::decode(&buf[readed..]).and_then(|(bytes_read, k)| {
        readed += bytes_read;

        V::decode(&buf[readed..]).map(|(bytes_read, v)| {
          readed += bytes_read;
          (k, v)
        })
      }))
    } else {
      None
    }
  })
  .collect::<Result<O, _>>()
  .map(|output| (readed, output))
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
  ///
  /// ## Example
  ///
  /// ```rust
  /// use varing::EncodeError;
  ///
  /// let error = EncodeError::custom("Custom error message");
  /// assert_eq!(error.to_string(), "Custom error message");
  /// ```
  #[inline]
  pub const fn custom(msg: &'static str) -> Self {
    Self::Custom(msg)
  }

  #[inline]
  const fn update(self, required: usize, remaining: usize) -> Self {
    match self {
      Self::Underflow { .. } => Self::Underflow {
        required,
        remaining,
      },
      Self::Custom(msg) => Self::Custom(msg),
    }
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

/// An iterator that decodes a sequence of varint values from a buffer.
#[derive(Debug)]
pub struct SequenceDecoder<'a, V: ?Sized> {
  buf: &'a [u8],
  offset: usize,
  _m: core::marker::PhantomData<V>,
}

impl<V: ?Sized> Clone for SequenceDecoder<'_, V> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<V: ?Sized> Copy for SequenceDecoder<'_, V> {}

impl<'a, V: ?Sized> SequenceDecoder<'a, V> {
  #[inline]
  const fn new(src: &'a [u8]) -> Self {
    Self {
      buf: src,
      offset: 0,
      _m: core::marker::PhantomData,
    }
  }

  /// Returns the current position of the buffer.
  #[inline]
  pub const fn position(&self) -> usize {
    self.offset
  }
}

impl<V: Varint> Iterator for SequenceDecoder<'_, V> {
  type Item = Result<(usize, V), DecodeError>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.offset < self.buf.len() {
      match V::decode(&self.buf[self.offset..]) {
        Ok((bytes_read, value)) => {
          self.offset += bytes_read;
          Some(Ok((bytes_read, value)))
        }
        Err(e) => Some(Err(e)),
      }
    } else {
      None
    }
  }
}

/// An iterator that decodes a sequence of varint values from a buffer.
#[derive(Debug)]
pub struct MapDecoder<'a, K: ?Sized, V: ?Sized> {
  buf: &'a [u8],
  offset: usize,
  #[allow(clippy::type_complexity)]
  _m: core::marker::PhantomData<(fn() -> &'a K, fn() -> &'a V)>,
}

impl<K: ?Sized, V: ?Sized> Clone for MapDecoder<'_, K, V> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<K: ?Sized, V: ?Sized> Copy for MapDecoder<'_, K, V> {}

impl<'a, K: ?Sized, V: ?Sized> MapDecoder<'a, K, V> {
  #[inline]
  const fn new(src: &'a [u8]) -> Self {
    Self {
      buf: src,
      offset: 0,
      _m: core::marker::PhantomData,
    }
  }

  /// Returns the current position of the buffer.
  #[inline]
  pub const fn position(&self) -> usize {
    self.offset
  }
}

impl<K: Varint, V: Varint> Iterator for MapDecoder<'_, K, V> {
  type Item = Result<(usize, (K, V)), DecodeError>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.offset < self.buf.len() {
      let offset = self.offset;
      Some(
        K::decode(&self.buf[self.offset..]).and_then(|(bytes_read, k)| {
          self.offset += bytes_read;

          V::decode(&self.buf[self.offset..]).map(|(bytes_read, v)| {
            self.offset += bytes_read;
            (self.offset - offset, (k, v))
          })
        }),
      )
    } else {
      None
    }
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
