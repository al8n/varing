#![allow(warnings)]

use super::*;

use quickcheck::{Arbitrary, Gen};

#[derive(Clone, Copy, Debug)]
struct ArbitraryComplex<T> {
  re: T,
  im: T,
}

macro_rules! impl_arbitrary_complex {
    ($($ty:ty),+$(,)?) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< Complex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
    (@floats $($ty:ty),+$(,)?) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              loop {
                let re = <$ty>::arbitrary(g);
                let im = <$ty>::arbitrary(g);
                if re.is_nan() || im.is_nan() {
                  continue;
                } else {
                  return Self { re, im }
                }
              }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< Complex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
    (@ruint ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< RUintComplex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
    (@bnum ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< BnumComplex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
  }

impl_arbitrary_complex!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
impl_arbitrary_complex!(@floats f32, f64);

fuzzy!(@const_varint_into (ComplexU8(Complex<u8>), ComplexU16(Complex<u16>), ComplexU32(Complex<u32>), ComplexU64(Complex<u64>), ComplexI8(Complex<i8>), ComplexI16(Complex<i16>), ComplexI32(Complex<i32>), ComplexI64(Complex<i64>), ComplexF32(Complex<f32>), ComplexF64(Complex<f64>),));

fuzzy!(@varint_into (
  ComplexU8(Complex<u8>),
  ComplexU16(Complex<u16>),
  ComplexU32(Complex<u32>),
  ComplexU64(Complex<u64>),
  ComplexI8(Complex<i8>),
  ComplexI16(Complex<i16>),
  ComplexI32(Complex<i32>),
  ComplexI64(Complex<i64>),
  ComplexF32(Complex<f32>),
  ComplexF64(Complex<f64>),
));

#[cfg(feature = "ruint_1")]
mod ruint_1;

#[cfg(feature = "bnum_0_13")]
mod bnum_0_13;

// F6: `MIN_ENCODED_LEN` is the *minimum* encoded length of any value. The shortest
// value for the packed `Complex` types is `Complex { 0, 0 }` (packs to `0`), so its
// `encoded_len()` must fall within `MIN_ENCODED_LEN..=MAX_ENCODED_LEN`.
#[test]
fn min_encoded_len_in_range() {
  fn check<T: crate::Varint>(v: T) {
    let len = v.encoded_len().get();
    assert!(
      len >= T::MIN_ENCODED_LEN.get(),
      "encoded_len {len} below MIN_ENCODED_LEN {}",
      T::MIN_ENCODED_LEN.get(),
    );
    assert!(
      len <= T::MAX_ENCODED_LEN.get(),
      "encoded_len {len} above MAX_ENCODED_LEN {}",
      T::MAX_ENCODED_LEN.get(),
    );
    assert!(T::MIN_ENCODED_LEN.get() <= T::MAX_ENCODED_LEN.get());
  }

  // small and wide widths, unsigned and signed
  check(Complex { re: 0u8, im: 0u8 });
  check(Complex { re: 0u64, im: 0u64 });
  check(Complex { re: 0i8, im: 0i8 });
  check(Complex { re: 0i64, im: 0i64 });
  // float components delegate to the integer path
  check(Complex {
    re: 0.0f32,
    im: 0.0f32,
  });
  check(Complex {
    re: 0.0f64,
    im: 0.0f64,
  });
}
