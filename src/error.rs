use core::num::NonZeroUsize;

/// An error that occurs when trying to decode data from a buffer with insufficient data.
///
/// This error indicates that a decode operation failed because the buffer does not have
/// enough remaining bytes to decode the value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct InsufficientData {
  /// The number of bytes needed to decode the value, if known.
  required: Option<NonZeroUsize>,
  /// The number of bytes available.
  available: usize,
}

impl InsufficientData {
  /// Creates a new `InsufficientData` error with only the available bytes.
  #[inline]
  pub const fn new(available: usize) -> Self {
    Self {
      required: None,
      available,
    }
  }

  /// Creates a new `InsufficientData` error with the required and available bytes.
  ///
  /// # Panics
  ///
  /// Panics if `required` is not greater than `available`.
  #[inline]
  pub const fn with_required(required: NonZeroUsize, available: usize) -> Self {
    assert!(
      required.get() > available,
      "InsufficientData: required must be greater than available"
    );

    Self {
      required: Some(required),
      available,
    }
  }

  /// Returns the number of bytes required to decode the value, if known.
  #[inline]
  pub const fn required(&self) -> Option<NonZeroUsize> {
    self.required
  }

  /// Returns the number of bytes available in the buffer.
  #[inline]
  pub const fn available(&self) -> usize {
    self.available
  }
}

impl core::fmt::Display for InsufficientData {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self.required {
      Some(required) => write!(
        f,
        "not enough bytes to decode value (required {} but only {} available)",
        required, self.available
      ),
      None => write!(
        f,
        "not enough bytes to decode value (only {} available)",
        self.available
      ),
    }
  }
}

impl core::error::Error for InsufficientData {}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<InsufficientData> for std::io::Error {
  fn from(err: InsufficientData) -> Self {
    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, err)
  }
}

/// An error that occurs when trying to write data to a buffer with insufficient space.
///
/// This error indicates that a write operation failed because the buffer does not have
/// enough remaining capacity to hold the data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[error(
  "not enough space available to encode value (requested {requested} but only {available} available)"
)]
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
  /// Panics if `requested` is not greater than `available`.
  #[inline]
  pub const fn new(requested: NonZeroUsize, available: usize) -> Self {
    assert!(
      requested.get() > available,
      "InsufficientSpace: requested must be greater than available"
    );

    Self {
      requested,
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
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<InsufficientSpace> for std::io::Error {
  fn from(err: InsufficientSpace) -> Self {
    std::io::Error::new(std::io::ErrorKind::WriteZero, err)
  }
}

/// Encode varint error
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ConstEncodeError {
  /// The buffer does not have enough capacity to encode the value.
  #[error(transparent)]
  InsufficientSpace(#[from] InsufficientSpace),
  /// A custom error message.
  #[error("{0}")]
  Other(&'static str),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<ConstEncodeError> for std::io::Error {
  fn from(err: ConstEncodeError) -> Self {
    match err {
      ConstEncodeError::InsufficientSpace(err) => {
        std::io::Error::new(std::io::ErrorKind::WriteZero, err)
      }
      ConstEncodeError::Other(msg) => std::io::Error::other(msg),
    }
  }
}

impl ConstEncodeError {
  /// Creates a new `ConstEncodeError::InsufficientSpace` with the requested and available bytes.
  ///
  /// # Panics
  ///
  /// Panics if `requested` is not greater than `available`.
  #[inline]
  pub const fn insufficient_space(requested: NonZeroUsize, available: usize) -> Self {
    Self::InsufficientSpace(InsufficientSpace::new(requested, available))
  }

  /// Creates a new `ConstEncodeError::Other` with the given message.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use varing::ConstEncodeError;
  ///
  /// let error = ConstEncodeError::other("Other error message");
  /// assert_eq!(error.to_string(), "Other error message");
  /// ```
  #[inline]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
  }

  #[inline]
  pub(super) const fn update(self, requested: NonZeroUsize, available: usize) -> Self {
    match self {
      Self::InsufficientSpace(_) => {
        Self::InsufficientSpace(InsufficientSpace::new(requested, available))
      }
      Self::Other(msg) => Self::Other(msg),
    }
  }

  /// Converts this `ConstEncodeError` into an `EncodeError`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use varing::{ConstEncodeError, EncodeError};
  /// use core::num::NonZeroUsize;
  ///
  /// let const_error = ConstEncodeError::other("Other error message");
  /// let error: EncodeError = const_error.into_encode_error();
  ///
  /// let const_error_insufficient = ConstEncodeError::insufficient_space(NonZeroUsize::new(10).unwrap(), 5);
  /// let error_insufficient: EncodeError = const_error_insufficient.into_encode_error();
  /// ```
  #[inline]
  pub const fn into_encode_error(self) -> EncodeError {
    match self {
      Self::InsufficientSpace(iss) => EncodeError::InsufficientSpace(iss),
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(msg) => EncodeError::Other(std::borrow::Cow::Borrowed(msg)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      Self::Other(msg) => EncodeError::Other(msg),
    }
  }
}

/// Decoding varint error.
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ConstDecodeError {
  /// The buffer does not contain a valid LEB128 encoding.
  #[error("decoded value would overflow the target type")]
  Overflow,
  /// The buffer does not contain enough data to decode.
  #[error(transparent)]
  InsufficientData(#[from] InsufficientData),
  /// A custom error message.
  #[error("{0}")]
  Other(&'static str),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<ConstDecodeError> for std::io::Error {
  fn from(err: ConstDecodeError) -> Self {
    match err {
      ConstDecodeError::Overflow => std::io::Error::new(std::io::ErrorKind::InvalidData, err),
      ConstDecodeError::InsufficientData(err) => {
        std::io::Error::new(std::io::ErrorKind::UnexpectedEof, err)
      }
      ConstDecodeError::Other(msg) => std::io::Error::other(msg),
    }
  }
}

impl ConstDecodeError {
  /// Creates a new `ConstDecodeError::Overflow` indicating that the decoded value would overflow the target type.
  #[inline]
  pub const fn overflow() -> Self {
    Self::Overflow
  }

  /// Creates a new `ConstDecodeError::InsufficientData` indicating that the buffer does not have enough data
  /// to decode a value.
  #[inline]
  pub const fn insufficient_data(available: usize) -> Self {
    Self::InsufficientData(InsufficientData::new(available))
  }

  /// Creates a new `ConstDecodeError::InsufficientData` with the required and available bytes.
  ///
  /// # Panics
  ///
  /// Panics if `required` is not greater than `available`.
  #[inline]
  pub const fn insufficient_data_with_required(required: NonZeroUsize, available: usize) -> Self {
    Self::InsufficientData(InsufficientData::with_required(required, available))
  }

  /// Creates a new `ConstDecodeError::Other` with the given message.
  #[inline]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
  }

  /// Converts this `ConstDecodeError` into a `DecodeError`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use varing::{ConstDecodeError, DecodeError};
  ///
  /// let const_error = ConstDecodeError::other("Other error message");
  /// let error: DecodeError = const_error.into_decode_error();
  ///
  /// let const_error_overflow = ConstDecodeError::overflow();
  /// let error_overflow: DecodeError = const_error_overflow.into_decode_error();
  ///
  /// let const_error_insufficient = ConstDecodeError::insufficient_data(5);
  /// let error_insufficient: DecodeError = const_error_insufficient.into_decode_error();
  /// ```
  #[inline]
  pub const fn into_decode_error(self) -> DecodeError {
    match self {
      Self::Overflow => DecodeError::Overflow,
      Self::InsufficientData(e) => DecodeError::InsufficientData(e),
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(msg) => DecodeError::Other(std::borrow::Cow::Borrowed(msg)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      Self::Other(msg) => DecodeError::Other(msg),
    }
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
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[error("{0}")]
  Other(&'static str),
  /// A custom error message.
  #[error("{0}")]
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(std::borrow::Cow<'static, str>),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<EncodeError> for std::io::Error {
  fn from(err: EncodeError) -> Self {
    match err {
      EncodeError::InsufficientSpace(err) => {
        std::io::Error::new(std::io::ErrorKind::WriteZero, err)
      }
      EncodeError::Other(msg) => std::io::Error::other(msg),
    }
  }
}

impl From<ConstEncodeError> for EncodeError {
  fn from(err: ConstEncodeError) -> Self {
    match err {
      ConstEncodeError::InsufficientSpace(iss) => EncodeError::InsufficientSpace(iss),
      ConstEncodeError::Other(msg) => EncodeError::other(msg),
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
  pub const fn insufficient_space(requested: NonZeroUsize, available: usize) -> Self {
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
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
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
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(msg: impl Into<std::borrow::Cow<'static, str>>) -> Self {
    Self::Other(msg.into())
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
  #[error(transparent)]
  InsufficientData(#[from] InsufficientData),
  /// A custom error message.
  #[error("{0}")]
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  Other(&'static str),
  /// A custom error message.
  #[error("{0}")]
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(std::borrow::Cow<'static, str>),
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<DecodeError> for std::io::Error {
  fn from(err: DecodeError) -> Self {
    match err {
      DecodeError::Overflow => std::io::Error::new(std::io::ErrorKind::InvalidData, err),
      DecodeError::InsufficientData(err) => {
        std::io::Error::new(std::io::ErrorKind::UnexpectedEof, err)
      }
      DecodeError::Other(msg) => std::io::Error::other(msg),
    }
  }
}

impl From<ConstDecodeError> for DecodeError {
  fn from(err: ConstDecodeError) -> Self {
    match err {
      ConstDecodeError::Overflow => Self::Overflow,
      ConstDecodeError::InsufficientData(e) => Self::InsufficientData(e),
      ConstDecodeError::Other(msg) => Self::other(msg),
    }
  }
}

impl DecodeError {
  /// Creates a new `DecodeError::Overflow` indicating that the decoded value would overflow the target type.
  #[inline]
  pub const fn overflow() -> Self {
    Self::Overflow
  }

  /// Creates a new `DecodeError::InsufficientData` indicating that the buffer does not have enough data
  /// to decode a value.
  #[inline]
  pub const fn insufficient_data(available: usize) -> Self {
    Self::InsufficientData(InsufficientData::new(available))
  }

  /// Creates a new `DecodeError::InsufficientData` with the required and available bytes.
  ///
  /// # Panics
  ///
  /// Panics if `required` is not greater than `available`.
  #[inline]
  pub const fn insufficient_data_with_required(required: NonZeroUsize, available: usize) -> Self {
    Self::InsufficientData(InsufficientData::with_required(required, available))
  }

  /// Creates a new `DecodeError::Other` with the given message.
  #[inline]
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
  }

  /// Creates a new `DecodeError::Other` with the given message.
  #[inline]
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(msg: impl Into<std::borrow::Cow<'static, str>>) -> Self {
    Self::Other(msg.into())
  }
}
