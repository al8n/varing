pub use super::packable::builtin::*;

/// A read-only buffer for storing LEB128 encoded values.
#[derive(Debug, Copy, Clone, Eq)]
pub struct Buffer<const N: usize>([u8; N]);

impl<const N: usize> PartialEq for Buffer<N> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.as_slice().eq(other.as_slice())
  }
}

impl<const N: usize> core::hash::Hash for Buffer<N> {
  #[inline]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state)
  }
}

impl<const N: usize> PartialOrd for Buffer<N> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<const N: usize> Ord for Buffer<N> {
  #[inline]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.as_slice().cmp(other.as_slice())
  }
}

impl<const N: usize> AsRef<[u8]> for Buffer<N> {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    self
  }
}

impl<const N: usize> core::ops::Deref for Buffer<N> {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    let len = self.0[N - 1] as usize;
    &self.0[..len]
  }
}

impl<const N: usize> core::borrow::Borrow<[u8]> for Buffer<N> {
  #[inline]
  fn borrow(&self) -> &[u8] {
    self
  }
}

impl<const N: usize> Buffer<N> {
  pub(crate) const CAPACITY: usize = N - 1;

  pub(crate) const fn new(buffer: [u8; N]) -> Self {
    Self(buffer)
  }

  /// Returns the length of buffer.
  #[inline]
  pub const fn len(&self) -> usize {
    self.0[Self::CAPACITY] as usize
  }

  /// Returns `true` if the buffer is empty.
  #[inline]
  pub const fn is_empty(&self) -> bool {
    self.0[Self::CAPACITY] == 0
  }

  /// Returns the buffer as a slice.
  #[inline]
  pub const fn as_slice(&self) -> &[u8] {
    let len = self.len();
    let (data, _) = self.0.split_at(len);
    data
  }
}

/// Zigzag encode `i8` value.
#[inline]
pub const fn zigzag_encode_i8(value: i8) -> u8 {
  ((value << 1) ^ (value >> 7)) as u8
}

/// Zigzag encode `i16` value.
#[inline]
pub const fn zigzag_encode_i16(value: i16) -> u16 {
  ((value << 1) ^ (value >> 15)) as u16
}

/// Zigzag encode `i32` value.
#[inline]
pub const fn zigzag_encode_i32(value: i32) -> u32 {
  ((value << 1) ^ (value >> 31)) as u32
}

/// Zigzag encode `i64` value.
#[inline]
pub const fn zigzag_encode_i64(value: i64) -> u64 {
  ((value << 1) ^ (value >> 63)) as u64
}

/// Zigzag encode `i128` value.
#[inline]
pub const fn zigzag_encode_i128(value: i128) -> u128 {
  ((value << 1) ^ (value >> 127)) as u128
}

/// Zigzag decode `i8` value.
#[inline]
pub const fn zigzag_decode_i8(value: u8) -> i8 {
  ((value >> 1) as i8) ^ (-((value & 1) as i8))
}

/// Zigzag decode `i16` value.
#[inline]
pub const fn zigzag_decode_i16(value: u16) -> i16 {
  ((value >> 1) as i16) ^ (-((value & 1) as i16))
}

/// Zigzag decode `i32` value.
#[inline]
pub const fn zigzag_decode_i32(value: u32) -> i32 {
  ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

/// Zigzag decode `i64` value.
#[inline]
pub const fn zigzag_decode_i64(value: u64) -> i64 {
  ((value >> 1) as i64) ^ (-((value & 1) as i64))
}

/// Zigzag decode `i128` value.
#[inline]
pub const fn zigzag_decode_i128(value: u128) -> i128 {
  ((value >> 1) as i128) ^ (-((value & 1) as i128))
}
