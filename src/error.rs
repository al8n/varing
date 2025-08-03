use core::num::NonZeroUsize;

/// An error that occurs when trying to write data to a buffer with insufficient space.
///
/// This error indicates that a write operation failed because the buffer does not have
/// enough remaining capacity to hold the data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not enough space available to encode value (requested {requested} but only {available} available)")]
pub struct InsufficientSpace {
  /// The number of bytes needed to encode the value.
  requested: NonZeroUsize,
  /// The number of bytes available.
  available: usize,
}

impl InsufficientSpace {
  /// Creates a new `InsufficientSpace` error with the requested and available bytes.
  ///
  /// # Panics
  ///
  /// Panics if `requested` is not greater than `available` or if `requested` is zero.
  #[inline]
  pub const fn new(requested: usize, available: usize) -> Self {
    debug_assert!(
      requested > available,
      "InsufficientSpace: requested must be greater than available"
    );

    Self {
      requested: NonZeroUsize::new(requested)
        .expect("InsufficientSpace: requested must be non-zero"),
      available,
    }
  }

  /// Returns the number of bytes requested to encode the value.
  #[inline]
  pub const fn requested(&self) -> NonZeroUsize {
    self.requested
  }

  /// Returns the number of bytes available in the buffer.
  #[inline]
  pub const fn available(&self) -> usize {
    self.available
  }

  /// Returns the number of additional bytes needed for the operation to succeed.
  ///
  /// This is equivalent to `requested() - available()`.
  #[inline]
  pub const fn shortage(&self) -> usize {
    self.requested - self.available
  }
}

/// Encode varint error
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeError {
  /// The buffer does not have enough capacity to encode the value.
  #[error(transparent)]
  InsufficientSpace(#[from] InsufficientSpace),
  /// A custom error message.
  #[error("{0}")]
  Other(&'static str),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<EncodeError> for std::io::Error {
  fn from(err: EncodeError) -> Self {
    match err {
      EncodeError::InsufficientSpace(_) => std::io::Error::new(std::io::ErrorKind::WriteZero, err),
      EncodeError::Other(msg) => std::io::Error::other(msg),
    }
  }
}

impl EncodeError {
  /// Creates a new `EncodeError::InsufficientSpace` with the requested and available bytes.
  ///
  /// # Panics
  ///
  /// Panics if `requested` is not greater than `available` or if `requested` is zero.
  #[inline]
  pub const fn insufficient_space(requested: usize, available: usize) -> Self {
    Self::InsufficientSpace(InsufficientSpace::new(requested, available))
  }

  /// Creates a new `EncodeError::Other` with the given message.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use varing::EncodeError;
  ///
  /// let error = EncodeError::other("Other error message");
  /// assert_eq!(error.to_string(), "Other error message");
  /// ```
  #[inline]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
  }

  #[inline]
  pub(super) const fn update(self, requested: usize, available: usize) -> Self {
    match self {
      Self::InsufficientSpace(_) => {
        Self::InsufficientSpace(InsufficientSpace::new(requested, available))
      }
      Self::Other(msg) => Self::Other(msg),
    }
  }
}

/// Decoding varint error.
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum DecodeError {
  /// The buffer does not contain a valid LEB128 encoding.
  #[error("decoded value would overflow the target type")]
  Overflow,
  /// The buffer does not contain enough data to decode.
  #[error("not enough data available to decode value")]
  InsufficientData,
  /// A custom error message.
  #[error("{0}")]
  Other(&'static str),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<DecodeError> for std::io::Error {
  fn from(err: DecodeError) -> Self {
    match err {
      DecodeError::Overflow => std::io::Error::new(std::io::ErrorKind::InvalidData, err),
      DecodeError::InsufficientData => std::io::Error::new(std::io::ErrorKind::UnexpectedEof, err),
      DecodeError::Other(msg) => std::io::Error::other(msg),
    }
  }
}

impl DecodeError {
  /// Creates a new `DecodeError::Other` with the given message.
  #[inline]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
  }
}
