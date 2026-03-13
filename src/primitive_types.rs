/// Implementations of `Varint` for the `primitive-types` version `0.13` crate.
#[cfg(feature = "primitive-types_0_13")]
#[cfg_attr(docsrs, doc(cfg(feature = "primitive-types_0_13")))]
pub mod v0_13;

/// Implements varint encoding and decoding for types from the `primitive-types` version `0.14` crate.
#[cfg(feature = "primitive-types_0_14")]
#[cfg_attr(docsrs, doc(cfg(feature = "primitive-types_0_14")))]
pub mod v0_14;
