/// Encode varint error
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeError {
  /// The buffer does not have enough capacity to encode the value.
  #[error("Not enough bytes available to write value (requested {requested} but only {available} available)")]
  InsufficientSpace {
    /// The number of bytes needed to encode the value.
    requested: usize,
    /// The number of bytes available.
    available: usize,
  },
  /// A custom error message.
  #[error("{0}")]
  Custom(&'static str),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<EncodeError> for std::io::Error {
  fn from(err: EncodeError) -> Self {
    match err {
      EncodeError::InsufficientSpace { .. } => {
        std::io::Error::new(std::io::ErrorKind::WriteZero, err)
      }
      EncodeError::Custom(msg) => std::io::Error::other(msg),
    }
  }
}

impl EncodeError {
  /// Creates a new `EncodeError::InsufficientSpace` with the requested and available bytes.
  #[inline]
  pub const fn insufficient_space(requested: usize, available: usize) -> Self {
    Self::InsufficientSpace {
      requested,
      available,
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
  const fn update(self, requested: usize, available: usize) -> Self {
    match self {
      Self::InsufficientSpace { .. } => Self::InsufficientSpace {
        requested,
        available,
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
  #[error("Decoded value would overflow the target type")]
  Overflow,
  /// The buffer does not contain enough data to decode.
  #[error("Not enough data available to decode value")]
  InsufficientData,
  /// A custom error message.
  #[error("{0}")]
  Custom(&'static str),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<DecodeError> for std::io::Error {
  fn from(err: DecodeError) -> Self {
    match err {
      DecodeError::Overflow => std::io::Error::new(std::io::ErrorKind::InvalidData, err),
      DecodeError::InsufficientData => std::io::Error::new(std::io::ErrorKind::UnexpectedEof, err),
      DecodeError::Custom(msg) => std::io::Error::other(msg),
    }
  }
}

impl DecodeError {
  /// Creates a new `DecodeError::Custom` with the given message.
  #[inline]
  pub const fn custom(msg: &'static str) -> Self {
    Self::Custom(msg)
  }
}
