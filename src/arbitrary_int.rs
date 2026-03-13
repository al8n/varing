/// Implements varint encoding and decoding for arbitrary-precision integer types from the `arbitrary-int` version `1` crate.
#[cfg(feature = "arbitrary-int_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary-int_1")))]
pub mod v1;

/// Implements varint encoding and decoding for arbitrary-precision integer types from the `arbitrary-int` version `2` crate.
#[cfg(feature = "arbitrary-int_2")]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary-int_2")))]
pub mod v2;
