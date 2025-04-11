pub(crate) mod builtin;

/// Packable implementation for [`arbitrary-int`](https://crates.io/crates/arbitrary-int) types.
#[cfg(any(feature = "arbitrary-int_1", feature = "arbitrary-int"))]
#[cfg_attr(
  docsrs,
  doc(cfg(any(feature = "arbitrary-int_1", feature = "arbitrary-int")))
)]
pub mod arbitrary_int;

/// Packable implementation for [`bnum`](https://crates.io/crates/bnum) types.
#[cfg(any(feature = "bnum_0_13", feature = "bnum"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "bnum_0_13", feature = "bnum"))))]
pub mod bnum;

/// Packable implementation for [`ruint`](https://crates.io/crates/ruint) types.
#[cfg(any(feature = "ruint_1", feature = "ruint"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "ruint"))))]
pub mod ruint;

/// A trait for types that can be packed into a single value, which
/// can be used for varint encoding.
///
/// This trait is used to define how to pack and unpack values of
/// different types into a single value.
///
/// A common usage is to pack `Self` and `Rhs` into a single
/// value `Packed`, if the `Packed` implements [`Varint`](crate::Varint),
/// then the packed value can be used for varint encoding in one go.
/// However, the output size of the packed value is not guaranteed to be
/// smaller than seperate encoding of `Self` and `Rhs`, and most of the
/// time, the output size is larger than the sum of encoding two types separately.
///
/// The `Rhs` type is the type that will be packed with the current
/// type.
///
/// The `Packed` type is the type that will be used to store the
/// packed value.
pub trait Packable<Rhs: ?Sized, Packed> {
  /// Packs the current value and the given `rhs` into a single value.
  fn pack(&self, rhs: &Rhs) -> Packed;

  /// Unpacks the packed value into the current type and the given `rhs`.
  fn unpack(packed: Packed) -> (Self, Rhs)
  where
    Self: Sized,
    Rhs: Sized;
}
