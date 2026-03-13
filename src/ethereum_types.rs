/// Implements varint encoding and decoding for types from the `ethereum-types` version `0.15` crate.
#[cfg(feature = "ethereum-types_0_15")]
pub mod v0_15;

/// Implements varint encoding and decoding for types from the `ethereum-types` version `0.16` crate.
#[cfg(feature = "ethereum-types_0_16")]
pub mod v0_16;
