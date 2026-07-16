#![allow(warnings)]

use super::*;

use quickcheck::{Arbitrary, Gen};

#[derive(Clone, Copy, Debug)]
struct ArbitraryRatio<T> {
  numer: T,
  denom: T,
}

macro_rules! impl_arbitrary_ratio {
    ($($ty:ty),+$(,)?) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if denom != 0 {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< Ratio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
    (@ruint ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if !denom.is_zero() {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< RUintRatio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
    (@bnum ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if !denom.is_zero() {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< BnumRatio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
  }

impl_arbitrary_ratio!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);

fuzzy!(@const_varint_into (RatioU8(Ratio<u8>), RatioU16(Ratio<u16>), RatioU32(Ratio<u32>), RatioU64(Ratio<u64>), RatioI8(Ratio<i8>), RatioI16(Ratio<i16>), RatioI32(Ratio<i32>), RatioI64(Ratio<i64>),));
fuzzy!(@varint_into (
  RatioU8(Ratio<u8>),
  RatioU16(Ratio<u16>),
  RatioU32(Ratio<u32>),
  RatioU64(Ratio<u64>),
  RatioI8(Ratio<i8>),
  RatioI16(Ratio<i16>),
  RatioI32(Ratio<i32>),
  RatioI64(Ratio<i64>),
));

#[cfg(feature = "bnum_0_13")]
mod bnum_0_13;

#[cfg(feature = "ruint_1")]
mod ruint_1;

// F6: `MIN_ENCODED_LEN` is the *minimum* encoded length of any value. The shortest
// representable `Ratio` is a zero numerator over `1` (the smallest nonzero
// denominator). Because `pack` stores the numerator low and the denominator high,
// `0/1` does not pack to `0`, so `MIN_ENCODED_LEN` must equal `encoded_len(0/1)`
// (which is 2+ bytes), and no representable `Ratio` may encode any shorter.
#[test]
fn min_encoded_len_in_range() {
  use crate::Varint;

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

  // `MIN_ENCODED_LEN` must equal the encoded length of `0/1` for small and wide
  // widths, unsigned and signed.
  assert_eq!(
    Ratio::<u8>::MIN_ENCODED_LEN.get(),
    Ratio::new_raw(0u8, 1u8).encoded_len().get(),
  );
  assert_eq!(
    Ratio::<u64>::MIN_ENCODED_LEN.get(),
    Ratio::new_raw(0u64, 1u64).encoded_len().get(),
  );
  assert_eq!(
    Ratio::<i8>::MIN_ENCODED_LEN.get(),
    Ratio::new_raw(0i8, 1i8).encoded_len().get(),
  );
  assert_eq!(
    Ratio::<i64>::MIN_ENCODED_LEN.get(),
    Ratio::new_raw(0i64, 1i64).encoded_len().get(),
  );

  // small and wide widths, unsigned and signed
  check(Ratio::new_raw(0u8, 1u8));
  check(Ratio::new_raw(0u64, 1u64));
  check(Ratio::new_raw(0i8, 1i8));
  check(Ratio::new_raw(0i64, 1i64));

  // Non-vacuous: exhaust every representable `Ratio<u8>` and `Ratio<i8>`. `0/1`
  // achieves `MIN_ENCODED_LEN` and nothing encodes shorter.
  let min_u8 = Ratio::<u8>::MIN_ENCODED_LEN.get();
  assert_eq!(min_u8, Ratio::new_raw(0u8, 1u8).encoded_len().get());
  for numer in 0u8..=u8::MAX {
    for denom in 1u8..=u8::MAX {
      assert!(Ratio::new_raw(numer, denom).encoded_len().get() >= min_u8);
    }
  }
  let min_i8 = Ratio::<i8>::MIN_ENCODED_LEN.get();
  assert_eq!(min_i8, Ratio::new_raw(0i8, 1i8).encoded_len().get());
  for numer in i8::MIN..=i8::MAX {
    for denom in i8::MIN..=i8::MAX {
      if denom == 0 {
        continue;
      }
      assert!(Ratio::new_raw(numer, denom).encoded_len().get() >= min_i8);
    }
  }
}
